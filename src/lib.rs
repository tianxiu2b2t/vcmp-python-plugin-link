use std::{
    ffi::CStr,
    fmt::Display,
    sync::{Arc, OnceLock},
};

use libloading::Library;

use crate::raw::{PluginCallbacks, PluginFuncs, PluginInfo};

pub mod cfg;
#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod raw;

pub static LIBRARY: OnceLock<Arc<Library>> = OnceLock::new();
pub static ORIGIN_SERVER_INIT: OnceLock<Option<unsafe extern "C" fn() -> u8>> = OnceLock::new();
pub static PY_VERSION: OnceLock<PyVersion> = OnceLock::new();
const VERSION: &str = env!("CARGO_PKG_VERSION");
const PROJECT_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug, Clone)]
pub enum PyVersion {
    Py3_10,
    Py3_11,
    Py3_12,
    Py3_13,
    Unknown(String),
}

impl From<String> for PyVersion {
    fn from(version: String) -> Self {
        let mut split_ver = version.split_once(" ").unwrap().0.split(".");
        let major = split_ver.next().unwrap().parse::<u8>().unwrap();
        let minor = split_ver.next().unwrap().parse::<u8>().unwrap();
        match (major, minor) {
            (3, 10) => PyVersion::Py3_10,
            (3, 11) => PyVersion::Py3_11,
            (3, 12) => PyVersion::Py3_12,
            (3, 13) => PyVersion::Py3_13,
            _ => PyVersion::Unknown(version),
        }
    }
}
impl Display for PyVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PyVersion::Py3_10 => "3.10.x",
                PyVersion::Py3_11 => "3.11.x",
                PyVersion::Py3_12 => "3.12.x",
                PyVersion::Py3_13 => "3.13.x",
                PyVersion::Unknown(version) => version,
            }
        )
    }
}

impl PyVersion {
    pub fn to_version(&self) -> &'static str {
        match self {
            PyVersion::Py3_10 => "310",
            PyVersion::Py3_11 => "311",
            PyVersion::Py3_12 => "312",
            PyVersion::Py3_13 => "313",
            PyVersion::Unknown(_) => "unknown",
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn server_init() -> u8 {
    println!(
        "{PROJECT_NAME} loaded with version {VERSION} on Python {} by tianxiu2b2t",
        PY_VERSION.get().unwrap()
    );
    let origin = *ORIGIN_SERVER_INIT.get().unwrap();
    match origin {
        Some(origin) => unsafe { origin() },
        None => 1,
    }
}

#[unsafe(no_mangle)]
extern "C" fn VcmpPluginInit(
    plugin_functions: *mut PluginFuncs,
    plugin_callbacks: *mut PluginCallbacks,
    plugin_info: *mut PluginInfo,
) -> u32 {
    let py_version = PyVersion::from(unsafe {
        CStr::from_ptr(pyo3::ffi::Py_GetVersion())
            .to_string_lossy()
            .to_string()
    });

    PY_VERSION.set(py_version.clone()).unwrap();

    let version = match py_version {
        PyVersion::Unknown(version) => {
            println!("Unsupported Python version: {version}");
            return 0;
        }
        v => v.to_version(),
    };

    let cfg = match cfg::parse_cfg() {
        Err(e) => {
            println!("Failed to parse config: {e}");
            return 0;
        }
        Ok(cfg) => cfg,
    };

    let plugin_file = cfg.filename_format.replace("{py_version}", version);
    let plugin_path = cfg.dir.join(plugin_file);

    let res = unsafe {
        let lib = Arc::new(libloading::Library::new(plugin_path).unwrap());
        LIBRARY.set(lib.clone()).unwrap();
        let plugin_funcs: libloading::Symbol<
            fn(*mut PluginFuncs, *mut PluginCallbacks, *mut PluginInfo) -> u32,
        > = lib.get(b"VcmpPluginInit").unwrap();
        plugin_funcs(plugin_functions, plugin_callbacks, plugin_info)
    };

    // get plugin_callbacks
    if res == 1 {
        let callback = unsafe { &mut *plugin_callbacks };
        let origin = callback.OnServerInitialise.clone();
        ORIGIN_SERVER_INIT.set(origin).unwrap();
        callback.OnServerInitialise = Some(server_init);
    };
    res
}
