mod tracing;

use futures::{Stream, StreamExt};
use postgres_connector_types::transport::{TcpTransport, Transaction, Transport};
use worker::{wasm_bindgen::JsValue, *};

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    tracing::init();
    let url = env
        .get_binding::<HyperdriveBinding>("HYPERDRIVE")?
        .connection_string();

    let mut transport = TcpTransport::new(&url)
        .await
        .map_err(|err| err.to_string())?;

    {
        let mut rows = transport.parameterized_query("SELECT * FROM pg_tables", vec![]);

        worker::console_log!("Starting to stream new rows");
        while let Some(row) = rows.next().await {
            let row = row.map_err(|err| err.to_string())?;
            worker::console_log!("new row! {row:#?}");
        }
    }

    {
        let transaction = &mut transport;
        // let transaction = transport
        //     .transaction()
        //     .await
        //     .map_err(|err| err.to_string())?;
        worker::console_log!("Creating table");
        transaction.parameterized_query(
            "CREATE TABLE IF NOT EXIST values (id SERIAL PRIMARY KEY, value TEXT NOT NULL);",
            vec![],
        );

        worker::console_log!("Inserting");
        let rows = transaction
            .parameterized_query("INSERT INTO values (value) VALUES ($1,) ($2,) ($3,)", vec![]);
        let rows: Vec<_> = rows.collect::<Vec<_>>().await;
        for row in rows {
            row.map_err(|err| err.to_string())?;
        }

        worker::console_log!("Inserting again");
        let rows = transaction.parameterized_query(
            "INSERT INTO values  (value) VALUES ($1) ($2) ($3) ($4)",
            vec![],
        );
        let rows: Vec<_> = rows.collect::<Vec<_>>().await;
        for row in rows {
            row.map_err(|err| err.to_string())?;
        }
    }

    Response::ok(format!("Hi! hyperdrive url: {url:?}"))
}

pub struct HyperdriveBinding(JsValue);

impl HyperdriveBinding {
    pub fn connection_string(&self) -> String {
        worker::js_sys::Reflect::get(&self.0, &"connectionString".into())
            .ok()
            .and_then(|value| value.as_string())
            .expect("must be a string")
    }
}

impl EnvBinding for HyperdriveBinding {
    const TYPE_NAME: &'static str = "Hyperdrive";
}

impl worker::wasm_bindgen::JsCast for HyperdriveBinding {
    fn instanceof(_: &JsValue) -> bool {
        true
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        HyperdriveBinding(val)
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue).cast::<Self>() }
    }
}

impl AsRef<JsValue> for HyperdriveBinding {
    fn as_ref(&self) -> &JsValue {
        unsafe { &*std::ptr::addr_of!(self.0) }
    }
}

impl From<JsValue> for HyperdriveBinding {
    fn from(val: JsValue) -> Self {
        HyperdriveBinding(val)
    }
}

impl From<HyperdriveBinding> for JsValue {
    fn from(sec: HyperdriveBinding) -> Self {
        sec.0
    }
}
