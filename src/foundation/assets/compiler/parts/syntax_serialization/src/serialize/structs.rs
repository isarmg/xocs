use oxc_data_structures::code_buffer::CodeBuffer;

use super::{
    Config, ESTree, ESTreeSequenceSerializer, ESTreeSerializer, Formatter, Serializer,
    TracePathPart,
};

/// Trait for struct serializers.
pub trait StructSerializer {
    type Config: Config;
    type Formatter: Formatter;

    /// Serialize struct field.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    fn serialize_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T);

    /// Serialize struct field which is JS syntax only (not in TS AST).
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `include_ts_fields() == false`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `include_ts_fields() == true`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    fn serialize_js_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T);

    /// Serialize struct field which is TypeScript syntax.
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `include_ts_fields() == true`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `include_ts_fields() == false`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    fn serialize_ts_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T);

    /// Serialize `Span`.
    ///
    /// * If `serializer.ranges() == true`, outputs `start`, `end`, and `range` fields.
    /// * Otherwise, outputs only `start` and `end`.
    fn serialize_span<S: ESTreeSpan>(&mut self, span: S);

    /// Finish serializing struct.
    fn end(self);

    /// Get whether output should contain TS fields.
    fn include_ts_fields(&self) -> bool;

    /// Get whether output should contain `range` fields.
    fn ranges(&self) -> bool;
}

/// Serializer for structs.
///
/// This is returned by `ESTreeSerializer::serialize_struct`.
pub struct ESTreeStructSerializer<'s, C: Config, F: Formatter> {
    /// Serializer
    serializer: &'s mut ESTreeSerializer<C, F>,
    /// State of struct.
    /// Starts as `StructState::Empty`, transitions to `StructState::HasFields` on first field.
    state: StructState,
}

impl<'s, C: Config, F: Formatter> ESTreeStructSerializer<'s, C, F> {
    /// Create new [`ESTreeStructSerializer`].
    pub(super) fn new(mut serializer: &'s mut ESTreeSerializer<C, F>) -> Self {
        // Push item to `trace_path`. It will be replaced with a `TracePathPart::Key`
        // when serializing each field in the struct, and popped off again in `end` method.
        if serializer.config.fixes() {
            serializer.trace_path.push(TracePathPart::DUMMY);
        }

        serializer.buffer_mut().print_ascii_byte(b'{');

        Self { serializer, state: StructState::Empty }
    }
}

impl<C: Config, F: Formatter> StructSerializer for ESTreeStructSerializer<'_, C, F> {
    type Config = C;
    type Formatter = F;

    /// Serialize struct field.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    fn serialize_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        // Update last item in trace path to current key
        if self.serializer.config.fixes() {
            *self.serializer.trace_path.last_mut() = TracePathPart::Key(key);
        }

        let (buffer, formatter) = self.serializer.buffer_and_formatter_mut();
        if self.state == StructState::Empty {
            self.state = StructState::HasFields;
            formatter.before_first_element(buffer);
        } else {
            buffer.print_ascii_byte(b',');
            formatter.before_later_element(buffer);
        }

        buffer.print_strs_array(["\"", key, "\":"]);
        formatter.before_field_value(buffer);
        value.serialize(&mut *self.serializer);
    }

    /// Serialize struct field which is JS syntax only (not in TS AST).
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `include_ts_fields() == false`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `include_ts_fields() == true`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_js_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        if !self.include_ts_fields() {
            self.serialize_field(key, value);
        }
    }

    /// Serialize struct field which is TypeScript syntax.
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `include_ts_fields() == true`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `include_ts_fields() == false`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_ts_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        if self.include_ts_fields() {
            self.serialize_field(key, value);
        }
    }

    /// Serialize `Span`.
    ///
    /// * If `serializer.ranges() == true`, outputs `start`, `end`, and `range` fields.
    /// * Otherwise, outputs only `start` and `end`.
    fn serialize_span<S: ESTreeSpan>(&mut self, span: S) {
        let range = span.range();
        self.serialize_field("start", &range[0]);
        self.serialize_field("end", &range[1]);
        if self.serializer.ranges() {
            self.serialize_field("range", &range);
        }
    }

    /// Finish serializing struct.
    fn end(self) {
        let mut serializer = self.serializer;

        // Pop entry for this struct from `trace_path`
        if serializer.config.fixes() {
            // SAFETY: `trace_path` is pushed to in `new`, which is only way to create an `ESTreeStructSerializer`.
            // This method consumes the `ESTreeStructSerializer`, so this method can't be called more
            // times than `new`. So there must be an item to pop.
            unsafe { serializer.trace_path.pop_unchecked() };
        }

        let (buffer, formatter) = serializer.buffer_and_formatter_mut();
        if self.state == StructState::HasFields {
            formatter.after_last_element(buffer);
        }
        buffer.print_ascii_byte(b'}');
    }

    /// Get whether output should contain TS fields.
    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        self.serializer.include_ts_fields()
    }

    /// Get whether output should contain `range` fields.
    #[inline(always)]
    fn ranges(&self) -> bool {
        self.serializer.ranges()
    }
}

/// State of [`StructSerializer`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StructState {
    Empty,
    HasFields,
}

/// Flattening struct serializer.
///
/// The only method it's legal to call on a `FlatStructSerializer` is `serialize_struct`.
///
/// # Example
/// ```rust,ignore
/// struct SomeNode {
///     // We want to flatten `Span` as if its fields were on `SomeNode`
///     span: Span,
///     whatever: u32,
/// }
///
/// struct Span {
///     start: u32,
///     end: u32,
/// }
///
/// impl ESTree for SomeNode {
///     fn serialize<S: Serializer>(&self, serializer: S) {
///         let mut state = serializer.serialize_struct();
///         self.inner.serialize(FlatStructSerializer(&mut state));
///         state.serialize_field("whatever", &self.whatever);
///         state.end();
///     }
/// }
///
/// impl ESTree for Span {
///     fn serialize<S: Serializer>(&self, serializer: S) {
///         let mut state = serializer.serialize_struct();
///         state.serialize_field("start", &self.start);
///         state.serialize_field("end", &self.end);
///         state.end();
///     }
/// }
/// ```
pub struct FlatStructSerializer<'p, P: StructSerializer>(pub &'p mut P);

impl<'p, P: StructSerializer> Serializer for FlatStructSerializer<'p, P> {
    type Formatter = P::Formatter;
    type StructSerializer = Self;
    type SequenceSerializer = ESTreeSequenceSerializer<'p, P::Config, P::Formatter>;

    /// Serialize struct.
    fn serialize_struct(self) -> Self {
        self
    }

    fn serialize_sequence(self) -> ESTreeSequenceSerializer<'p, P::Config, P::Formatter> {
        const {
            panic!("Cannot flatten a sequence into a struct");
        }
    }

    fn record_fix_path(&mut self) {
        const {
            panic!("Cannot call `record_fix_path` on a `FlatStructSerializer`");
        }
    }

    /// Get whether output should contain TS fields.
    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        self.0.include_ts_fields()
    }

    /// Get whether output should contain `range` fields.
    #[inline(always)]
    fn ranges(&self) -> bool {
        self.0.ranges()
    }

    fn buffer_mut(&mut self) -> &mut CodeBuffer {
        const {
            panic!("Cannot flatten anything but a struct into another struct");
        }
    }

    fn buffer_and_formatter_mut(&mut self) -> (&mut CodeBuffer, &mut P::Formatter) {
        const {
            panic!("Cannot flatten anything but a struct into another struct");
        }
    }
}

impl<P: StructSerializer> StructSerializer for FlatStructSerializer<'_, P> {
    type Config = P::Config;
    type Formatter = P::Formatter;

    /// Serialize struct field.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        // Delegate to parent `StructSerializer`
        self.0.serialize_field(key, value);
    }

    /// Serialize struct field which is JS syntax only (not in TS AST).
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `include_ts_fields() == false`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `include_ts_fields() == true`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_js_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        // Delegate to parent `StructSerializer`
        self.0.serialize_js_field(key, value);
    }

    /// Serialize struct field which is TypeScript syntax.
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `include_ts_fields() == true`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `include_ts_fields() == false`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_ts_field<T: ESTree + ?Sized>(&mut self, key: &'static str, value: &T) {
        // Delegate to parent `StructSerializer`
        self.0.serialize_ts_field(key, value);
    }

    /// Serialize `Span`.
    ///
    /// * If `serializer.ranges() == true`, outputs `start`, `end`, and `range` fields.
    /// * Otherwise, outputs only `start` and `end`.
    fn serialize_span<S: ESTreeSpan>(&mut self, span: S) {
        self.0.serialize_span(span);
    }

    /// Finish serializing struct.
    fn end(self) {
        // No-op - there may be more fields to be added to the struct in the parent
    }

    /// Get whether output should contain TS fields.
    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        self.0.include_ts_fields()
    }

    /// Get whether output should contain `range` fields.
    #[inline(always)]
    fn ranges(&self) -> bool {
        self.0.ranges()
    }
}

/// Trait for `Span` to implement.
///
/// This is a workaround to avoid circular dependency. `oxc_span` crate depends on `oxc_estree` crate,
/// so we can't import `Span` directly from `oxc_span` here.
pub trait ESTreeSpan: Copy {
    fn range(self) -> [u32; 2];
}

#[path = "../../../../../../../../test/foundation/assets/compiler/unit/parts/syntax_serialization/src/serialize/structs/tests.rs"]
#[cfg(test)]
mod tests;
