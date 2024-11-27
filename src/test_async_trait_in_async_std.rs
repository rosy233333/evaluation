use core::{
    future::{poll_fn, Future}, pin::Pin, slice, task::{Context, Poll}
};

use async_trait::async_trait;
use stackfuture::StackFuture;

// -------- async-trait --------

#[async_trait]
pub trait AsyncTraitRead {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;
}

// -------- stackfuture --------

pub trait StackFutureRead {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> StackFuture<'a, Result<usize, Error>, 512>;
}

// -------- static dispatch --------

pub trait StaticDispatchRead {
    fn poll_read(self: Pin<&mut Self>, buf: &mut [u8], cx: &mut Context<'_>) -> Poll<Result<usize, Error>>;
}

impl<T: StaticDispatchRead + Unpin + ?Sized> StaticDispatchRead for &mut T {
    fn poll_read(mut self: Pin<&mut Self>, buf: &mut [u8], cx: &mut Context<'_>) -> Poll<Result<usize, Error>> {
        Pin::new(&mut **self).poll_read(buf, cx)
    }
}

pub trait StaticDispatchAsyncRead: StaticDispatchRead {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadFuture<'a, Self>
    where Self: Unpin {
        ReadFuture { reader: self, buf }
    }
}

pub struct ReadFuture<'a, T: Unpin + ?Sized> {
    reader: &'a mut T,
    buf: &'a mut [u8]
}

impl<T: StaticDispatchRead + Unpin + ?Sized> Future for ReadFuture<'_, T> {
    type Output = Result<usize, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { reader, buf } = &mut *self;
        Pin::new(reader).poll_read(buf, cx)
    }
}

impl<T> StaticDispatchAsyncRead for T where T: StaticDispatchRead + Unpin + ?Sized {}

// -------- AFIT static dispatch --------

// reuse the `StaticDispatchRead` trait

pub trait AfitStaticDispatchAsyncRead: StaticDispatchRead {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>
    where Self: Unpin {
        let mut pinned = Pin::new(self);
        poll_fn(|cx| pinned.as_mut().poll_read(buf, cx)).await
    }
}

impl<T> AfitStaticDispatchAsyncRead for T where T: StaticDispatchRead + Unpin + ?Sized {}

// -------- dynosaur --------

#[dynosaur::dynosaur(DynosaurDynRead)]
pub trait DynosaurRead {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;
}

use async_std::{fs::File, io::WriteExt};
use async_std::io::{Error, ReadExt};

pub struct TestFile {
    pub path: String,
}

pub struct StaticDispatchTestFile {
    pub path: String,
    pub fut: Option<Pin<Box<dyn Future<Output = Result<usize, Error>>>>>,
}

async fn static_dispatch_future_read(path: String, buf_ptr: *mut u8, buf_len: usize) -> Result<usize, Error> {
    let mut file = File::open(path).await.unwrap();
    let buf = unsafe { slice::from_raw_parts_mut(buf_ptr, buf_len) };
    let res = file.read(buf).await;
    drop(file);
    res
}

// -------- async-trait --------

#[async_trait]
impl AsyncTraitRead for TestFile {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut file = File::open(&self.path).await.unwrap();
        let res = file.read(buf).await;
        drop(file);
        res
    }
}

// -------- stackfuture --------

impl StackFutureRead for TestFile {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> StackFuture<'a, Result<usize, Error>, 512> {
        StackFuture::from(async {
                    let mut file = File::open(&self.path).await.unwrap();
        let res = file.read(buf).await;
        drop(file);
        res
        })
    }
}

// -------- static dispatch --------

impl StaticDispatchRead for StaticDispatchTestFile {
    fn poll_read(mut self: Pin<&mut Self>, buf: &mut [u8], cx: &mut Context<'_>) -> Poll<Result<usize, Error>> {
        if self.fut.is_none() {
            // 将局部生命周期的变量传入future会无法通过编译。因此使用裸指针的形式传递buf
            let buf_ptr = buf.as_mut_ptr();
            let buf_len = buf.len();
            self.fut = Some(Box::pin(static_dispatch_future_read(self.path.clone(), buf_ptr, buf_len)));
        }
        let res = self.fut.as_mut().unwrap().as_mut().poll(cx);
        if res.is_ready() {
            self.fut = None;
        }
        res
    }
}

// -------- AFIT static dispatch --------

// Because `StaticDispatchRead` is reused, there's no need to implement new trait.

// -------- dynosaur --------

impl DynosaurRead for TestFile {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut file = File::open(&self.path).await.unwrap();
        let res = file.read(buf).await;
        drop(file);
        res
    }
}

#[async_std::test]
async fn test_async_read() {
    use std::time::Instant;
    const READ_TIMES: usize = 2000000;
    // const READ_TIMES: usize = 1;
    const BUF_SIZE: usize = 8;

    let mut file = TestFile {
        path: String::from("./foo.txt"),
    };
    let mut static_dispatch_file = StaticDispatchTestFile {
        path: String::from("./foo.txt"),
        fut: None,
    };
    let mut buf = [0u8; BUF_SIZE];

    // -------- async-trait --------

    let mut time_elapse = Vec::new();
    for _ in 0..READ_TIMES {
        let curr = Instant::now();
        AsyncTraitRead::read(&mut file, &mut buf).await;
        let elapse = Instant::now().duration_since(curr);
        time_elapse.push(elapse.as_nanos());
    }

    let mut async_trait_read_out = File::create("./async_trait_read_out.txt").await.unwrap();
    let mut res = format!("{:?}", time_elapse);
    res.remove(0);
    res.pop();
    let res_buf = res.as_bytes();
    async_trait_read_out.write_all(&res_buf).await.unwrap();

    // -------- stackfuture --------

    let mut time_elapse = Vec::new();
    for _ in 0..READ_TIMES {
        let curr = Instant::now();
        StackFutureRead::read(&mut file, &mut buf).await;
        let elapse = Instant::now().duration_since(curr);
        time_elapse.push(elapse.as_nanos());
    }

    let mut stack_future_read_out = File::create("./stack_future_read_out.txt").await.unwrap();
    let mut res = format!("{:?}", time_elapse);
    res.remove(0);
    res.pop();
    let res_buf = res.as_bytes();
    stack_future_read_out.write_all(&res_buf).await.unwrap();

    // -------- static dispatch --------

    let mut time_elapse = Vec::new();
    for _ in 0..READ_TIMES {
        let curr = Instant::now();
        StaticDispatchAsyncRead::read(&mut static_dispatch_file, &mut buf).await;
        let elapse = Instant::now().duration_since(curr);
        time_elapse.push(elapse.as_nanos());
    }

    let mut static_dispatch_read_out = File::create("./static_dispatch_read_out.txt").await.unwrap();
    let mut res = format!("{:?}", time_elapse);
    res.remove(0);
    res.pop();
    let res_buf = res.as_bytes();
    static_dispatch_read_out.write_all(&res_buf).await.unwrap();

    // -------- AFIT static dispatch --------

    let mut time_elapse = Vec::new();
    for _ in 0..READ_TIMES {
        let curr = Instant::now();
        AfitStaticDispatchAsyncRead::read(&mut static_dispatch_file, &mut buf).await;
        let elapse = Instant::now().duration_since(curr);
        time_elapse.push(elapse.as_nanos());
    }

    let mut afit_static_dispatch_read_out = File::create("./afit_static_dispatch_read_out.txt").await.unwrap();
    let mut res = format!("{:?}", time_elapse);
    res.remove(0);
    res.pop();
    let res_buf = res.as_bytes();
    afit_static_dispatch_read_out.write_all(&res_buf).await.unwrap();

    // -------- dynosaur --------

    let mut time_elapse = Vec::new();
    for _ in 0..READ_TIMES {
        let curr = Instant::now();
        DynosaurRead::read(DynosaurDynRead::from_mut(&mut file), &mut buf).await;
        let elapse = Instant::now().duration_since(curr);
        time_elapse.push(elapse.as_nanos());
    }

    let mut dynosaur_read_out = File::create("./dynosaur_read_out.txt").await.unwrap();
    let mut res = format!("{:?}", time_elapse);
    res.remove(0);
    res.pop();
    let res_buf = res.as_bytes();
    dynosaur_read_out.write_all(&res_buf).await.unwrap();
}
