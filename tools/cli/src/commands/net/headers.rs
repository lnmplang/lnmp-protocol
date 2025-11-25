use anyhow::Result;
use clap::Args;
use lnmp_net::NetMessage;
use std::path::PathBuf;

use crate::utils::read_file;

#[derive(Args)]
pub struct HeadersCmd {
    /// Input network message file
    input: PathBuf,

    /// Transport protocol
    #[arg(long, value_enum)]
    transport: TransportType,

    /// Output format
    #[arg(long, value_enum, default_value = "plain")]
    format: OutputFormat,
}

#[derive(Clone, clap::ValueEnum)]
enum TransportType {
    Http,
    Kafka,
    Nats,
    Grpc,
}

#[derive(Clone, clap::ValueEnum)]
enum OutputFormat {
    Plain,
    Json,
    Curl,
    Env,
}

impl HeadersCmd {
    pub fn execute(&self) -> Result<()> {
        // Read network message
        let data = read_file(&self.input)?;
        let net_msg: NetMessage = bincode::deserialize(&data)?;

        match self.transport {
            TransportType::Http => self.generate_http_headers(&net_msg),
            TransportType::Kafka => self.generate_kafka_headers(&net_msg),
            TransportType::Nats => self.generate_nats_headers(&net_msg),
            TransportType::Grpc => self.generate_grpc_headers(&net_msg),
        }
    }

    fn generate_http_headers(&self, msg: &NetMessage) -> Result<()> {
        let headers = vec![
            ("X-LNMP-Kind", format!("{:?}", msg.kind)),
            ("X-LNMP-Priority", msg.priority.to_string()),
            ("X-LNMP-TTL", msg.ttl_ms.to_string()),
        ];

        let mut all_headers = headers;
        if let Some(ref class) = msg.class {
            all_headers.push(("X-LNMP-Class", class.clone()));
        }

        match self.format {
            OutputFormat::Plain => {
                println!("HTTP Headers:");
                for (key, value) in all_headers {
                    println!("{}: {}", key, value);
                }
            }
            OutputFormat::Json => {
                println!("{{");
                for (i, (key, value)) in all_headers.iter().enumerate() {
                    let comma = if i < all_headers.len() - 1 { "," } else { "" };
                    println!("  \"{}\": \"{}\"{}", key, value, comma);
                }
                println!("}}");
            }
            OutputFormat::Curl => {
                for (key, value) in all_headers {
                    println!("-H \"{}: {}\" \\", key, value);
                }
            }
            OutputFormat::Env => {
                for (key, value) in all_headers {
                    let env_key = key.replace("-", "_").to_uppercase();
                    println!("export {}=\"{}\"", env_key, value);
                }
            }
        }

        Ok(())
    }

    fn generate_kafka_headers(&self, msg: &NetMessage) -> Result<()> {
        let headers = vec![
            ("lnmp.kind", format!("{:?}", msg.kind)),
            ("lnmp.priority", msg.priority.to_string()),
            ("lnmp.ttl", msg.ttl_ms.to_string()),
        ];

        let mut all_headers = headers;
        if let Some(ref class) = msg.class {
            all_headers.push(("lnmp.class", class.clone()));
        }

        match self.format {
            OutputFormat::Plain => {
                println!("Kafka Headers:");
                for (key, value) in all_headers {
                    println!("{}: {}", key, value);
                }
            }
            OutputFormat::Json => {
                println!("{{");
                for (i, (key, value)) in all_headers.iter().enumerate() {
                    let comma = if i < all_headers.len() - 1 { "," } else { "" };
                    println!("  \"{}\": \"{}\"{}", key, value, comma);
                }
                println!("}}");
            }
            _ => {
                println!("Kafka Headers:");
                for (key, value) in all_headers {
                    println!("{}: {}", key, value);
                }
            }
        }

        Ok(())
    }

    fn generate_nats_headers(&self, msg: &NetMessage) -> Result<()> {
        let headers = vec![
            ("lnmp-kind", format!("{:?}", msg.kind)),
            ("lnmp-priority", msg.priority.to_string()),
            ("lnmp-ttl", msg.ttl_ms.to_string()),
        ];

        let mut all_headers = headers;
        if let Some(ref class) = msg.class {
            all_headers.push(("lnmp-class", class.clone()));
        }

        match self.format {
            OutputFormat::Plain => {
                println!("NATS Headers:");
                for (key, value) in all_headers {
                    println!("{}: {}", key, value);
                }
            }
            OutputFormat::Json => {
                println!("{{");
                for (i, (key, value)) in all_headers.iter().enumerate() {
                    let comma = if i < all_headers.len() - 1 { "," } else { "" };
                    println!("  \"{}\": \"{}\"{}", key, value, comma);
                }
                println!("}}");
            }
            _ => {
                println!("NATS Headers:");
                for (key, value) in all_headers {
                    println!("{}: {}", key, value);
                }
            }
        }

        Ok(())
    }

    fn generate_grpc_headers(&self, msg: &NetMessage) -> Result<()> {
        let headers = vec![
            ("lnmp-kind", format!("{:?}", msg.kind)),
            ("lnmp-priority", msg.priority.to_string()),
            ("lnmp-ttl", msg.ttl_ms.to_string()),
        ];

        let mut all_headers = headers;
        if let Some(ref class) = msg.class {
            all_headers.push(("lnmp-class", class.clone()));
        }

        match self.format {
            OutputFormat::Plain => {
                println!("gRPC Metadata:");
                for (key, value) in all_headers {
                    println!("{}: {}", key, value);
                }
            }
            OutputFormat::Json => {
                println!("{{");
                for (i, (key, value)) in all_headers.iter().enumerate() {
                    let comma = if i < all_headers.len() - 1 { "," } else { "" };
                    println!("  \"{}\": \"{}\"{}", key, value, comma);
                }
                println!("}}");
            }
            _ => {
                println!("gRPC Metadata:");
                for (key, value) in all_headers {
                    println!("{}: {}", key, value);
                }
            }
        }

        Ok(())
    }
}
