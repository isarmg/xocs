
    use super::*;
    use tokio::io::{repeat, AsyncReadExt, Repeat};
    use tokio_stream::{once, Once, StreamExt};

    #[tokio::test]
    async fn either_is_stream() {
        let mut either: Either<Once<u32>, Once<u32>> = Either::Left(once(1));

        assert_eq!(Some(1u32), either.next().await);
    }

    #[tokio::test]
    async fn either_is_async_read() {
        let mut buffer = [0; 3];
        let mut either: Either<Repeat, Repeat> = Either::Right(repeat(0b101));

        either.read_exact(&mut buffer).await.unwrap();
        assert_eq!(buffer, [0b101, 0b101, 0b101]);
    }
