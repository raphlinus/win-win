use winapi::um::winnt::HRESULT;

// TODO: more helpful formatting (hex for hresult)
#[derive(Debug)]
pub enum Error {
    RegisterClassFailed,
    Hresult(HRESULT),
}
