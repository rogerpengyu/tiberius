use indicatif::ProgressBar;
use once_cell::sync::Lazy;
use std::env;
use tiberius::numeric::Numeric;
use tiberius::{
    BulkLoadMetadata, Client, ColumnFlag, Config, IntoSql, TokenRow, TypeInfo, TypeLength,
};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use tracing::log::info;

static CONN_STR: Lazy<String> = Lazy::new(|| {
    env::var("TIBERIUS_TEST_CONNECTION_STRING").unwrap_or_else(|_| {
        "server=tcp:localhost,1433;IntegratedSecurity=true;TrustServerCertificate=true".to_owned()
    })
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Config::from_ado_string(&CONN_STR)?;

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp.compat_write()).await?;

    client
        .execute("DROP TABLE IF EXISTS bulk_test1", &[])
        .await?;
    info!("drop table");
    client
        .execute(
            r#"CREATE TABLE bulk_test1 (
                        id INT IDENTITY PRIMARY KEY,
                        null_bit bit NULL, 
                        nonnull_bit bit NOT NULL,
                        null_tinyint tinyint NULL,
                        nonnull_tinyint tinyint NOT NULL, 
                        null_smallint smallint NULL,
                        nonnull_smallint smallint NOT NULL, 
                        null_int int NULL, 
                        nonnull_int int NOT NULL,
                        null_big bigint NULL,
                        nonnull_bigint bigint NOT NULL,
                        null_float real NULL,
                        nonnull_float real NOT NULL,
                        null_numeric numeric NULL,
                        nonnull_numeric numeric NOT NULL)"#,
            &[],
        )
        .await?;
    info!("create table done");

    let mut req = client.bulk_insert_1("bulk_test1").await?;

    let count = 1000i32;

    let pb = ProgressBar::new(count as u64);

    info!("start loading data");
    for i in 0..1000 {
        let mut row = TokenRow::new();

        // null_bit
        let null_bit = [Some(true), None][i % 2];
        row.push(null_bit.into_sql());

        // nonnull_bit
        let nonnull_bit = false;
        row.push(nonnull_bit.into_sql());

        // null_tinyint
        let null_tinyint = [Some(23u8), None][i % 2];
        row.push(null_tinyint.into_sql());

        // nonnull_bit
        let nonnull_tinyint = 45u8;
        row.push(nonnull_tinyint.into_sql());

        // null_smallint
        let null_smallint = [Some(23i16), None][i % 2];
        row.push(null_smallint.into_sql());

        // nonnull_bit
        let nonnull_smallint = 45i16;
        row.push(nonnull_smallint.into_sql());

        // null_int
        let null_int = [Some(32), None][i % 2];
        row.push(null_int.into_sql());

        // nonnull_int
        let nonnull_int = 44;
        row.push(nonnull_int.into_sql());

        // null_bigint
        let null_bigint = [Some(32i64), None][i % 2];
        row.push(null_bigint.into_sql());

        // nonnull_bigint
        let nonnull_bigint = 44i64;
        row.push(nonnull_bigint.into_sql());

        // null_float
        let null_float = [Some(34f32), None][i % 2];
        row.push(null_float.into_sql());

        // nonnull_float
        let nonnull_float = 32f32;
        row.push(nonnull_float.into_sql());

        // null_numeric
        let null_numeric = [Some(Numeric::new_with_scale(12, 0)), None][i % 2];
        row.push(null_numeric.into_sql());

        // nonnull_numeric
        let nonnull_numeric = Numeric::new_with_scale(23, 0);
        row.push(nonnull_numeric.into_sql());

        req.send(row).await?;
        pb.inc(1);
    }

    pb.finish_with_message("waiting...");

    let res = req.finalize().await?;

    info!("{:?}", res);

    Ok(())
}
