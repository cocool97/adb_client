use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    pyadb_client::stub_info()?.generate()
}
