#![allow(dead_code)]
use peace_proto_build::{preset_attr::SERDE, ProtoBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder = ProtoBuilder::new("proto", "peace", "generated");

    builder.build("base", None);
    builder.build("frame.logs", None);
    builder.build("services.chat", None);
    builder.build("services.bancho", None);
    builder.build(
        "services.bancho_state",
        Some(&[(
            SERDE,
            &["UserData", "ConnectionInfo", "GetAllSessionsResponse"],
        )]),
    );
    builder.build(
        "services.geoip",
        Some(&[(
            SERDE,
            &[
                "IpAddress",
                "GeoipData",
                "Location",
                "Continent",
                "Country",
                "Region",
                "City",
            ],
        )]),
    );

    builder.build("services.signature", None);

    Ok(())
}
