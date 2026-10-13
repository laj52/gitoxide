use gix_error::{ErrorExt, Exn, ExnResult, Message, exn, types};

fn main() -> ExnResult<(), Message> {
    let error = gix_error::Error::from_error(std::io::Error::from(std::io::ErrorKind::Other));
    assert_eq!(error.metadata().count(), 0, "native errors have no diagnostic metadata");
    let exception: Exn = error.into_exn();
    let _: &exn::Frame = exception.frame();
    let _: types::ChainedError = exception.into_chain();
    let error = gix_error::message("details").with_input("input").raise();
    assert_eq!(error.metadata().count(), 1, "experimental metadata remains available");
    gix_error::ensure!(true, "this guard succeeds");
    gix_error::bail!("the experimental macro remains usable");
}
