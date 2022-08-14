use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;

use macros::generate_call;

////////////////////////////////////////////////////////////////////////////////
// wasm_bindgen key bindings
////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen(module = "/src/holochain_client_wrapper.js")]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = AdminWebsocket, js_name="connect")]
    async fn connect_admin_ws_js(url: String, timeout: Option<u32>) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_namespace = AppWebsocket, js_name="connect")]
    async fn connect_app_ws_js(url: String, timeout: Option<u32>) -> Result<JsValue, JsValue>;
}

////////////////////////////////////////////////////////////////////////////////
// SerializeToJsObj trait
////////////////////////////////////////////////////////////////////////////////

trait SerializeToJsObj {
    fn serialize_to_js_obj(self) -> JsValue;
}

impl SerializeToJsObj for JsValue {
    fn serialize_to_js_obj(self) -> JsValue {
        self
    }
}

impl SerializeToJsObj for u16 {
    fn serialize_to_js_obj(self) -> JsValue {
        self.into()
    }
}

impl SerializeToJsObj for String {
    fn serialize_to_js_obj(self) -> JsValue {
        self.into()
    }
}

impl<T: SerializeToJsObj> SerializeToJsObj for Option<T> {
    fn serialize_to_js_obj(self) -> JsValue {
        match self {
            None => JsValue::NULL,
            Some(v) => v.serialize_to_js_obj(),
        }
    }
}

impl<A: SerializeToJsObj, B: SerializeToJsObj> SerializeToJsObj for (A, B) {
    fn serialize_to_js_obj(self) -> JsValue {
        let (a, b) = self;
        let val = Array::new();
        let _ = val.push(&a.serialize_to_js_obj());
        let _ = val.push(&b.serialize_to_js_obj());
        val.dyn_into().expect("Array conversion to succeed")
    }
}

impl<T: SerializeToJsObj> SerializeToJsObj for Vec<T> {
    fn serialize_to_js_obj(self) -> JsValue {
        let val = Array::new();
        for e in self.into_iter().rev() {
            let _ = val.push(&e.serialize_to_js_obj());
        }
        val.dyn_into().expect("Array conversion to succeed")
    }
}

#[derive(Clone, Debug)]
pub struct DnaHash(JsValue);

impl SerializeToJsObj for DnaHash {
    fn serialize_to_js_obj(self) -> JsValue {
        let DnaHash(val) = self;
        val
    }
}

#[derive(Clone, Debug)]
pub struct AgentPk(JsValue);

impl SerializeToJsObj for AgentPk {
    fn serialize_to_js_obj(self) -> JsValue {
        let AgentPk(val) = self;
        val
    }
}

pub type CellId = (DnaHash, AgentPk);

#[derive(Clone, Debug)]
pub struct HashRoleProof {
    hash: DnaHash,
    role: String,
    membrane_proof: Option<String>,
}

impl SerializeToJsObj for HashRoleProof {
    fn serialize_to_js_obj(self) -> JsValue {
        let ret = move || -> Result<JsValue, JsValue> {
            let val: JsValue = Object::new().dyn_into()?;
            assert!(Reflect::set(
                &val,
                &JsValue::from_str("hash"),
                &self.hash.serialize_to_js_obj(),
            )?);
            assert!(Reflect::set(
                &val,
                &JsValue::from_str("role"),
                &self.role.serialize_to_js_obj(),
            )?);
            match self.membrane_proof {
                None => {}
                Some(mp) => {
                    assert!(Reflect::set(
                        &val,
                        &JsValue::from_str("membrane_proof"),
                        &mp.serialize_to_js_obj(),
                    )?);
                }
            };
            Ok(val)
        };
        ret().expect("operations to succeed")
    }
}

// TODO figure out why this doesn't work - unsatisfied trait bounds for String
// impl<T> SerializeToJsObj for T
// where
//     T: JsCast,
// {
//     fn serialize_to_js_obj(self) -> JsValue {
//         self.into()
//     }
// }

////////////////////////////////////////////////////////////////////////////////
// DeserializeFromJsObj trait
////////////////////////////////////////////////////////////////////////////////

trait DeserializeFromJsObj {
    fn deserialize_from_js_obj(_: JsValue) -> Self;
}

impl<A: DeserializeFromJsObj, B: DeserializeFromJsObj> DeserializeFromJsObj for (A, B) {
    fn deserialize_from_js_obj(v: JsValue) -> Self {
        let arr: Array = v.dyn_into().expect("Array conversion to succeed");
        let a = arr.at(0);
        let b = arr.at(1);
        (A::deserialize_from_js_obj(a), B::deserialize_from_js_obj(b))
    }
}

impl DeserializeFromJsObj for AgentPk {
    fn deserialize_from_js_obj(v: JsValue) -> Self {
        Self(v)
    }
}

impl DeserializeFromJsObj for DnaHash {
    fn deserialize_from_js_obj(v: JsValue) -> Self {
        Self(v)
    }
}

////////////////////////////////////////////////////////////////////////////////
// AdminWebsocket
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct AdminWebsocket {
    pub js_ws: JsValue,
}

impl From<AdminWebsocket> for JsValue {
    fn from(ws: AdminWebsocket) -> Self {
        ws.js_ws
    }
}

pub async fn connect_admin_ws(url: String, timeout: Option<u32>) -> Result<AdminWebsocket, String> {
    match connect_admin_ws_js(url, timeout).await {
        Ok(js_ws) => Ok(AdminWebsocket { js_ws }),
        Err(js_err) => Err(format!("{:?}", js_err)),
    }
}

/// each constructor of this enum corresponds to a method on the AdminWebsocket:
/// <https://github.com/holochain/holochain-client-js/blob/develop/docs/API_adminwebsocket.md>
///
/// n.b. the order of the constructors is non-alphabetical & corresponds to the documentation
/// order.
#[generate_call(
    AdminWebsocket,
    AdminWsCmd,
    AdminWsCmdResponse,
    parse_admin_ws_cmd_response
)]
#[derive(Clone, Debug)]
pub enum AdminWsCmd {
    AttachAppInterface {
        port: u16,
    },
    DisableApp {
        installed_app_id: String,
    },
    // DumpState({ cell_id }),
    EnableApp {
        installed_app_id: String,
    },
    GenerateAgentPubKey,
    RegisterDna {
        path: String,
        uid: Option<String>,
        properties: Option<String>,
    },
    // InstallAppBundle({ installed_app_id, source as path | bundle | hash, uid?, properties? }),
    InstallApp {
        installed_app_id: String,
        agent_key: AgentPk,
        dnas: Vec<HashRoleProof>,
    },
    UninstallApp {
        installed_app_id: String,
    },
    ListDnas,
    ListCellIds,
    ListActiveApps,
    // RequestAgentInfo({ cell_id }),
    // AddAgentInfo({ agent_infos }),
}

// TODO consider statically checking that AdminWsCmd/AdminWsCmdResponse have the right # of
// constructors and all their names match up. can also apply to AppWsCmd/AppWsCmdResponse.

#[derive(Clone, Debug)]
pub enum AdminWsCmdResponse {
    AttachAppInterface(JsValue),
    DisableApp(JsValue),
    // DumpState(JsValue),
    EnableApp(JsValue),
    GenerateAgentPubKey(AgentPk),
    RegisterDna(DnaHash),
    // InstallAppBundle(JsValue),
    InstallApp(JsValue),
    UninstallApp(JsValue),
    ListDnas(JsValue),
    ListCellIds(CellId),
    ListActiveApps(JsValue),
    // RequestAgentInfo(JsValue),
    // AddAgentInfo(JsValue),
}

fn parse_admin_ws_cmd_response(val: JsValue, tag: String) -> AdminWsCmdResponse {
    match tag.as_str() {
        "AttachAppInterface" => AdminWsCmdResponse::AttachAppInterface(val),
        "DisableApp" => AdminWsCmdResponse::DisableApp(val),
        // "DumpState" => AdminWsCmdResponse::DumpState(val),
        "EnableApp" => AdminWsCmdResponse::EnableApp(val),
        "GenerateAgentPubKey" => {
            AdminWsCmdResponse::GenerateAgentPubKey(AgentPk::deserialize_from_js_obj(val))
        }
        "RegisterDna" => AdminWsCmdResponse::RegisterDna(DnaHash::deserialize_from_js_obj(val)),
        // "InstallAppBundle" => AdminWsCmdResponse::InstallAppBundle(val),
        "InstallApp" => AdminWsCmdResponse::InstallApp(val),
        "UninstallApp" => AdminWsCmdResponse::UninstallApp(val),
        "ListDnas" => AdminWsCmdResponse::ListDnas(val),
        "ListCellIds" => AdminWsCmdResponse::ListCellIds(CellId::deserialize_from_js_obj(val)),
        "ListActiveApps" => AdminWsCmdResponse::ListActiveApps(val),
        // "RequestAgentInfo" => AdminWsCmdResponse::RequestAgentInfo(val),
        // "AddAgentInfo" => AdminWsCmdResponse::AddAgentInfo(val),
        other => panic!(
            "parse_admin_ws_cmd_response: impossible: received unknown tag: {}",
            other
        ),
    }
}

////////////////////////////////////////
// payloads
////////////////////////////////////////

// this might be a good idea, but we'll leave it for later, maybe.
// pub struct RegisterDnaPayload {
//     bundle_src: BundleSource,
//     uid: Option<String>,
//     properties: Option<String>,
// }

// pub enum BundleSource {
//     Path(String),
//     // Hash(String),
//     // Bundle { ... },
// }

////////////////////////////////////////////////////////////////////////////////
// AppWebsocket
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct AppWebsocket {
    pub js_ws: JsValue,
}

impl From<AppWebsocket> for JsValue {
    fn from(ws: AppWebsocket) -> Self {
        ws.js_ws
    }
}

pub async fn connect_app_ws(url: String, timeout: Option<u32>) -> Result<AppWebsocket, String> {
    match connect_app_ws_js(url, timeout).await {
        Ok(js_ws) => Ok(AppWebsocket { js_ws }),
        Err(js_err) => Err(format!("{:?}", js_err)),
    }
}

/// n.b. the order of the constructors is non-alphabetical & corresponds to the order documented in
/// <https://github.com/holochain/holochain-client-js/blob/develop/docs/API_appwebsocket.md>
#[generate_call(AppWebsocket, AppWsCmd, AppWsCmdResponse, parse_app_ws_cmd_response)]
#[derive(Clone, Debug)]
pub enum AppWsCmd {
    AppInfo {
        installed_app_id: String,
    },
    CallZome {
        cell_id: CellId,
        zome_name: String,
        fn_name: String,
        payload: JsValue,
        provenance: AgentPk,
        cap: String,
    },
}

#[derive(Clone, Debug)]
pub enum AppWsCmdResponse {
    AppInfo(JsValue),
    CallZome(JsValue),
}

fn parse_app_ws_cmd_response(val: JsValue, tag: String) -> AppWsCmdResponse {
    match tag.as_str() {
        "AppInfo" => AppWsCmdResponse::AppInfo(val),
        "CallZome" => AppWsCmdResponse::CallZome(val),
        other => panic!(
            "parse_app_ws_cmd_response: impossible: received unknown tag: {}",
            other
        ),
    }
}

////////////////////////////////////////////////////////////////////////////////
// ZomeCallable
////////////////////////////////////////////////////////////////////////////////

trait ZomeCallable {
    type Input;
    type Output;

    const FN_NAME: str;

    fn prep_input(i: Self::Input) -> JsValue
    where
        Self::Input: SerializeToJsObj,
    {
        i.serialize_to_js_obj()
    }
    fn parse_output(v: JsValue) -> Self::Output
    where
        Self::Output: DeserializeFromJsObj,
    {
        Self::Output::deserialize_from_js_obj(v)
    }
}
