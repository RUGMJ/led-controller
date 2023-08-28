use crate::{Args, ClientType, WebStatus};
use zbus::{dbus_proxy, Connection};

#[dbus_proxy(
    interface = "dev.rugmj.LedController1",
    default_service = "dev.rugmj.LedController",
    default_path = "/dev/rugmj/LedController"
)]

trait Controller {
    async fn set_effect(&self, client_type: ClientType) -> zbus::Result<()>;
    async fn end_daemon(&self) -> zbus::Result<()>;
}

pub async fn controller(args: Args) -> zbus::Result<()> {
    let connection = Connection::session().await?;

    let proxy = ControllerProxy::new(&connection).await?;

    if let Some(client_type) = args.effect {
        println!("Setting client type to {:?}", client_type);
        proxy.set_effect(client_type).await?;
    }

    if args.kill {
        proxy.end_daemon().await.unwrap_err();
    }

    if let Some(web_status) = args.web_status {
        let web_status = match web_status {
            WebStatus::On => "true",
            WebStatus::Off => "false",
        };
        let url = format!("https://led.rugmj.dev/api/enabled/{}", web_status);
        reqwest::get(url).await.ok();
    }

    Ok(())
}
