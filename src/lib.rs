/// OpenRTB2 Serialization Models
#[allow(clippy::all)]
pub mod ortb {
    include!(concat!(env!("OUT_DIR"), "/com.iabtechlab.openrtb.v2.rs"));
    include!(concat!(
        env!("OUT_DIR"),
        "/com.iabtechlab.openrtb.v2.serde.rs"
    ));
}

pub mod listener;

#[cfg(test)]
mod tests {}
