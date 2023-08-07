#![allow(dead_code)]
use peace_proto_build::{preset_attr::SERDE, ProtoBuilder, StructAttr};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder = ProtoBuilder::new("proto", "peace", "generated");

    if let Err(err) = build_all(builder) {
        println!("---> !!!!!! [peace_pb ERROR] failed to build all protos: \"{err}\"");

        return Err(err);
    }

    Ok(())
}

fn build_all(builder: ProtoBuilder) -> Result<(), Box<dyn std::error::Error>> {
    builder.build("base")?;
    builder.build("frame.logs")?;
    builder.build("services.chat")?;
    builder.build("services.bancho")?;
    builder.build_with_attrs(
        "services.bancho_state",
        &[StructAttr::new(
            SERDE,
            &["UserData", "ConnectionInfo", "GetAllSessionsResponse"],
        )],
    )?;
    builder.build_with_attrs(
        "services.geoip",
        &[StructAttr::new(
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
        )],
    )?;

    builder.build("services.signature")?;
    builder.build("services.events")?;

    Ok(())
}
