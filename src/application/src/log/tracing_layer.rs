//! Tracing adapter for existing product instrumentation. Unstructured legacy
//! messages and internal error chains do not cross the ordinary-log boundary.

use crate::log::{CommonEvent, Level, LogError, LogRecord, identifier};
use serde_json::{Map, Value};
use std::{
    fmt::{self, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
use tracing::collector::{
    layer::{Context, Layer},
    registry::LookupSpan,
};
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id, Record},
};

const MAX_FIELDS: usize = 32;
const MAX_FIELD_BYTES: usize = 2048;

#[derive(Clone)]
pub struct BlogStructuredLayer {
    service: Arc<str>,
    rejected: Arc<AtomicU64>,
    output: Arc<Mutex<Sink>>,
}

enum Sink {
    Process,
    Stream(Box<dyn std::io::Write + Send>),
    #[cfg(any(unix, windows))]
    File(crate::log::RotatingLogFile),
}

impl BlogStructuredLayer {
    pub fn new(service: &str) -> Result<Self, LogError> {
        identifier(service)?;
        Ok(Self {
            service: service.into(),
            rejected: Arc::default(),
            output: Arc::new(Mutex::new(Sink::Process)),
        })
    }
    pub fn with_writer(mut self, output: impl std::io::Write + Send + 'static) -> Self {
        self.output = Arc::new(Mutex::new(Sink::Stream(Box::new(output))));
        self
    }

    #[cfg(any(unix, windows))]
    pub fn with_rotating_file(mut self, output: crate::log::RotatingLogFile) -> Self {
        self.output = Arc::new(Mutex::new(Sink::File(output)));
        self
    }

    /// Switch the installed subscriber's shared sink after the run preflight.
    /// All clones observe the switch; an unavailable sink is explicit.
    #[cfg(any(unix, windows))]
    pub fn set_rotating_file(&self, output: crate::log::RotatingLogFile) -> Result<(), LogError> {
        let mut sink = self.output.lock().map_err(|_| LogError::UnsafeStorage)?;
        *sink = Sink::File(output);
        Ok(())
    }

    pub fn rejected_count(&self) -> u64 {
        self.rejected.load(Ordering::Relaxed)
    }
}

#[derive(Default)]
struct Captured {
    fields: Map<String, Value>,
    truncated: bool,
}
impl Captured {
    fn insert(&mut self, name: &str, value: Value) {
        // Legacy text can contain SQL, credentials and error chains. The event
        // identity, level and safe attributes carry its useful classification.
        if name == "message" {
            return;
        }
        if name.len() > 128 || (!self.fields.contains_key(name) && self.fields.len() >= MAX_FIELDS)
        {
            self.truncated = true;
            return;
        }
        let value = if [
            "error",
            "source",
            "error_chain",
            "backtrace",
            "stack",
            "config",
        ]
        .contains(&name)
        {
            Value::String("[internal diagnostic omitted]".into())
        } else {
            value
        };
        self.fields.insert(name.into(), value);
    }
    fn merge(&mut self, other: &Self) {
        self.truncated |= other.truncated;
        for (key, value) in &other.fields {
            self.insert(key, value.clone());
        }
    }
    fn text(&mut self, name: &str) -> Option<String> {
        self.fields
            .remove(name)
            .and_then(|value| value.as_str().map(str::to_owned))
    }
}

struct FieldBuffer {
    text: String,
    truncated: bool,
}
impl Write for FieldBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let remaining = MAX_FIELD_BYTES.saturating_sub(self.text.len());
        if value.len() <= remaining {
            self.text.push_str(value);
            return Ok(());
        }
        let mut cut = remaining;
        while !value.is_char_boundary(cut) {
            cut -= 1;
        }
        self.text.push_str(&value[..cut]);
        self.truncated = true;
        Err(fmt::Error)
    }
}

impl Visit for Captured {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            return;
        }
        let mut buffer = FieldBuffer {
            text: String::new(),
            truncated: false,
        };
        let _ = write!(&mut buffer, "{value:?}");
        self.truncated |= buffer.truncated;
        self.insert(field.name(), Value::String(buffer.text));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        let mut buffer = FieldBuffer {
            text: String::new(),
            truncated: false,
        };
        let _ = buffer.write_str(value);
        self.truncated |= buffer.truncated;
        self.insert(field.name(), Value::String(buffer.text));
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.insert(field.name(), value.into());
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.insert(field.name(), value.into());
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.insert(field.name(), value.into());
    }
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.insert(field.name(), Value::from(value));
    }
}

impl<S> Layer<S> for BlogStructuredLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        let mut captured = Captured::default();
        attributes.record(&mut captured);
        if let Some(span) = context.span(id) {
            span.extensions_mut().insert(captured);
        }
    }
    fn on_record(&self, id: &Id, values: &Record<'_>, context: Context<'_, S>) {
        if let Some(span) = context.span(id)
            && let Some(captured) = span.extensions_mut().get_mut::<Captured>()
        {
            values.record(captured);
        }
    }
    fn on_event(&self, event: &Event<'_>, context: Context<'_, S>) {
        let mut captured = Captured::default();
        if let Some(scope) = context.event_scope(event) {
            for span in scope.from_root() {
                if let Some(fields) = span.extensions().get::<Captured>() {
                    captured.merge(fields);
                }
            }
        }
        event.record(&mut captured);
        let result = build_record(&self.service, event.metadata(), captured).and_then(|record| {
            let mut output = self.output.lock().map_err(|_| LogError::UnsafeStorage)?;
            match &mut *output {
                Sink::Process => record.emit(),
                Sink::Stream(output) => record.write_to(output),
                #[cfg(any(unix, windows))]
                Sink::File(output) => output.write(&record),
            }
        });
        if result.is_err() {
            self.rejected.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn build_record(
    service: &str,
    metadata: &tracing::Metadata<'_>,
    mut captured: Captured,
) -> Result<LogRecord, LogError> {
    let level = match *metadata.level() {
        tracing::Level::TRACE => Level::Trace,
        tracing::Level::DEBUG => Level::Debug,
        tracing::Level::INFO => Level::Info,
        tracing::Level::WARN => Level::Warn,
        tracing::Level::ERROR => Level::Error,
    };
    let component = captured
        .text("component")
        .unwrap_or_else(|| metadata.target().to_owned());
    let event = captured
        .text("event")
        .unwrap_or_else(|| format!("{service}.diagnostic"));
    let instance = captured.text("instance_id");
    let instance_type = captured.text("instance_type");
    let request = captured.text("request_id");
    let task = captured.text("task_id");
    let error = captured.text("error_code");
    let mut record = if event.starts_with("common.") {
        let common = CommonEvent::from_name(&event).ok_or(LogError::InvalidRecord)?;
        if let Some(instance) = instance {
            LogRecord::common_instance(service, common, &instance)?
        } else {
            LogRecord::common(service, common)?
        }
    } else if let Some(instance) = instance {
        LogRecord::instance(
            service,
            &component,
            &event,
            "Runtime diagnostic event.",
            level,
            &instance,
        )?
    } else {
        LogRecord::server(
            service,
            &component,
            &event,
            "Runtime diagnostic event.",
            level,
        )?
    };
    if let Some(instance_type) = instance_type {
        record = record.with_instance_type(&instance_type)?;
    }
    if let Some(request) = request {
        record = record.with_request_id(&request)?;
    }
    if let Some(task) = task {
        record = record.with_task_id(&task)?;
    }
    if let Some(error) = error {
        record = record.with_error_code(&error)?;
    }
    if captured.truncated {
        record = record.with_attribute("fields_truncated", true)?;
    }
    for (key, value) in captured.fields {
        record = record.with_attribute(&key, value)?;
    }
    Ok(record)
}

#[path = "../../../../test/application/unit/log/tracing_layer/tests.rs"]
#[cfg(test)]
mod tests;
