use super::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(inline_js = r#"
export function fakeDevice(mode) {
  const records = [];
  let releaseClaim;
  const adb = {alternateSetting:3, interfaceClass:255, interfaceSubclass:66, interfaceProtocol:1,
    endpoints:[{endpointNumber:5,direction:'in',type:'bulk',packetSize:64},
               {endpointNumber:7,direction:'out',type:'bulk',packetSize:64}]};
  const fastboot = {...adb, interfaceProtocol:3};
  const configuration = {configurationValue:2, interfaces:[
    {interfaceNumber:0,alternates:[fastboot]}, {interfaceNumber:1,alternates:[adb]}]};
  return {opened:mode==='opened', configuration:null, configurations:[configuration], records,
    async open() { records.push('open'); this.opened=true; },
    async selectConfiguration(value) { records.push('config:'+value); this.configuration=configuration; },
    async claimInterface(value) {
      records.push('claim:'+value);
      if (mode==='claim-error') throw new Error('interface busy');
      if (mode==='claim-pending') await new Promise(resolve => releaseClaim=resolve);
    },
    releaseClaim() { releaseClaim(); },
    async selectAlternateInterface(iface,alt) { records.push('alternate:'+iface+':'+alt); },
    async transferOut(endpoint,bytes) {
      records.push('write:'+endpoint+':'+bytes.byteLength);
      return {status:mode==='stall'?'stall':'ok',bytesWritten:bytes.byteLength-(mode==='short'?1:0)};
    },
    async transferIn(endpoint,length) {
      records.push('read:'+endpoint+':'+length);
      const bytes=new Uint8Array([99,1,2,3,88]);
      return {status:mode==='stall'?'stall':'ok',data:new DataView(bytes.buffer,1,3)};
    },
    async close() { records.push('close'); this.opened=false; }
  };
}
export function calls(device) { return device.records.join(','); }
export function releaseClaim(device) { device.releaseClaim(); }
"#)]
extern "C" {
    #[wasm_bindgen(js_name = fakeDevice)]
    fn fake_device(mode: &str) -> JsValue;
    fn calls(device: &JsValue) -> String;
    #[wasm_bindgen(js_name = releaseClaim)]
    fn release_claim(device: &JsValue);
}

async fn tick() {
    let _ = JsFuture::from(Promise::resolve(&JsValue::UNDEFINED)).await;
}

#[wasm_bindgen_test(async)]
async fn discovers_adb_not_fastboot_copies_dataview_offset_and_sends_zlp() {
    let device: UsbDevice = fake_device("ok").unchecked_into();
    let mut transport = WebUSBTransport::open(device.clone()).await.unwrap();
    let mut bytes = [0; 16];
    assert_eq!(transport.read(&mut bytes).await.unwrap(), 3);
    assert_eq!(&bytes[..3], &[1, 2, 3]);
    transport.write(&[0; 64]).await.unwrap();
    transport.close().await.unwrap();
    assert_eq!(
        calls(device.as_ref()),
        "open,config:2,claim:1,alternate:1:3,read:5:16,write:7:64,write:7:0,close"
    );
}

#[wasm_bindgen_test(async)]
async fn transfer_failures_propagate_and_claim_failure_closes_owned_device() {
    for mode in ["short", "stall"] {
        let device: UsbDevice = fake_device(mode).unchecked_into();
        let mut transport = WebUSBTransport::open(device.clone()).await.unwrap();
        assert!(transport.write(&[0; 64]).await.is_err());
        transport.close().await.unwrap();
        assert!(!calls(device.as_ref()).contains("write:7:0"));
    }
    let device: UsbDevice = fake_device("claim-error").unchecked_into();
    assert!(WebUSBTransport::open(device.clone()).await.is_err());
    tick().await;
    assert!(calls(device.as_ref()).ends_with("close"));
    let device: UsbDevice = fake_device("opened").unchecked_into();
    assert!(WebUSBTransport::open(device.clone()).await.is_err());
    tick().await;
    assert_eq!(
        calls(device.as_ref()),
        "",
        "do not close another client's device"
    );
}

#[wasm_bindgen_test(async)]
async fn cancelled_setup_closes_after_the_pending_claim_settles() {
    use std::{
        future::Future,
        task::{Context, Waker},
    };
    let device: UsbDevice = fake_device("claim-pending").unchecked_into();
    let mut open = Box::pin(WebUSBTransport::open(device.clone()));
    for _ in 0..10 {
        assert!(
            open.as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        tick().await;
        if calls(device.as_ref()).contains("claim:1") {
            break;
        }
    }
    assert!(calls(device.as_ref()).ends_with("claim:1"));
    drop(open);
    tick().await;
    assert!(!calls(device.as_ref()).contains("close"));
    release_claim(device.as_ref());
    for _ in 0..5 {
        tick().await;
    }
    assert!(calls(device.as_ref()).ends_with("close"));
}
