use std::os::raw::c_void;

use rglua::{
    lua::lua_pushlightuserdata,
    prelude::{LuaState, lua_newuserdata, lua_touserdata},
};

pub fn write_userdata<T>(l: LuaState, data: T) {
    let data_ptr = lua_newuserdata(l, std::mem::size_of::<T>()) as *mut T;
    unsafe {
        std::ptr::write(data_ptr, data);
    }
}

pub fn read_userdata_at<T: Clone>(l: LuaState, idx: i32) -> Result<T, String> {
    let data_ptr = lua_touserdata(l, idx) as *mut T;
    if data_ptr.is_null() {
        Err("Invalid userdata.".to_string())
    } else {
        Ok(unsafe { (*data_ptr).clone() })
    }
}

pub fn read_userdata<T: Clone>(l: LuaState) -> Result<T, String> {
    let data_ptr = lua_touserdata(l, 1) as *mut T;
    if data_ptr.is_null() {
        Err("Invalid userdata.".to_string())
    } else {
        Ok(unsafe { (*data_ptr).clone() })
    }
}

pub fn write_userdata_owned<T>(l: LuaState, data: T) {
    let boxed = Box::new(data);
    let raw_ptr = Box::into_raw(boxed);
    lua_pushlightuserdata(l, raw_ptr as *mut c_void);
}

pub fn read_userdata_owned<T>(l: LuaState, idx: i32) -> Result<Box<T>, String> {
    let ptr = lua_touserdata(l, idx) as *mut T;
    if ptr.is_null() {
        Err("Invalid userdata.".to_string())
    } else {
        Ok(unsafe { Box::from_raw(ptr) })
    }
}
