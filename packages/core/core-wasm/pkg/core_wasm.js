/* @ts-self-types="./core_wasm.d.ts" */

/**
 * @param {number} server_id
 * @param {string} channel
 * @param {string} before_msgid
 * @returns {Promise<History>}
 */
export function chat_channel_history_before(server_id, channel, before_msgid) {
  const ptr0 = passStringToWasm0(channel, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len0 = WASM_VECTOR_LEN
  const ptr1 = passStringToWasm0(before_msgid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len1 = WASM_VECTOR_LEN
  const ret = wasm.chat_channel_history_before(server_id, ptr0, len0, ptr1, len1)
  return ret
}

/**
 * @param {number} server_id
 * @param {string} channel
 * @param {string | null} [password]
 * @returns {Promise<Channel>}
 */
export function chat_channel_join(server_id, channel, password) {
  const ptr0 = passStringToWasm0(channel, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len0 = WASM_VECTOR_LEN
  var ptr1 = isLikeNone(password) ? 0 : passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  var len1 = WASM_VECTOR_LEN
  const ret = wasm.chat_channel_join(server_id, ptr0, len0, ptr1, len1)
  return ret
}

/**
 * @param {number} server_id
 * @param {string} channel_name
 * @param {string} text
 * @returns {Promise<Message>}
 */
export function chat_channel_send_message(server_id, channel_name, text) {
  const ptr0 = passStringToWasm0(channel_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len0 = WASM_VECTOR_LEN
  const ptr1 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len1 = WASM_VECTOR_LEN
  const ret = wasm.chat_channel_send_message(server_id, ptr0, len0, ptr1, len1)
  return ret
}

export function init() {
  wasm.init()
}

/**
 * @returns {Promise<Server[]>}
 */
export function initialize_orbit() {
  const ret = wasm.initialize_orbit()
  return ret
}

/**
 * @param {number} server_id
 * @returns {Promise<ChannelInfo[]>}
 */
export function server_channel_list(server_id) {
  const ret = wasm.server_channel_list(server_id)
  return ret
}

/**
 * @param {string} url
 * @returns {Promise<Server>}
 */
export function server_connect(url) {
  const ptr0 = passStringToWasm0(url, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len0 = WASM_VECTOR_LEN
  const ret = wasm.server_connect(ptr0, len0)
  return ret
}

/**
 * @param {number} server_id
 * @param {(event: ServerEvent) => void} f
 * @returns {Promise<void>}
 */
export function server_on_data(server_id, f) {
  const ret = wasm.server_on_data(server_id, f)
  return ret
}

/**
 * @param {number} server_id
 * @param {(event: string) => void} f
 * @returns {Promise<void>}
 */
export function server_on_disconnect(server_id, f) {
  const ret = wasm.server_on_disconnect(server_id, f)
  return ret
}

/**
 * @param {number} server_id
 * @param {(error: OrbitError) => void} f
 * @returns {Promise<void>}
 */
export function server_on_error(server_id, f) {
  const ret = wasm.server_on_error(server_id, f)
  return ret
}

/**
 * @param {number} server_id
 * @param {string} nick
 * @param {string} user
 * @param {string} realname
 * @param {string} password
 * @returns {Promise<SignedIn>}
 */
export function server_sign_in(server_id, nick, user, realname, password) {
  const ptr0 = passStringToWasm0(nick, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len0 = WASM_VECTOR_LEN
  const ptr1 = passStringToWasm0(user, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len1 = WASM_VECTOR_LEN
  const ptr2 = passStringToWasm0(realname, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len2 = WASM_VECTOR_LEN
  const ptr3 = passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len3 = WASM_VECTOR_LEN
  const ret = wasm.server_sign_in(server_id, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3)
  return ret
}

/**
 * @param {number} server_id
 * @param {string} nick
 * @param {string} user
 * @param {string} realname
 * @returns {Promise<SignedIn>}
 */
export function server_sign_in_anonymous(server_id, nick, user, realname) {
  const ptr0 = passStringToWasm0(nick, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len0 = WASM_VECTOR_LEN
  const ptr1 = passStringToWasm0(user, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len1 = WASM_VECTOR_LEN
  const ptr2 = passStringToWasm0(realname, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
  const len2 = WASM_VECTOR_LEN
  const ret = wasm.server_sign_in_anonymous(server_id, ptr0, len0, ptr1, len1, ptr2, len2)
  return ret
}
function __wbg_get_imports() {
  const import0 = {
    __proto__: null,
    __wbg_Error_92b29b0548f8b746: function (arg0, arg1) {
      const ret = Error(getStringFromWasm0(arg0, arg1))
      return ret
    },
    __wbg_Number_9a4e0ecb0fa16705: function (arg0) {
      const ret = Number(arg0)
      return ret
    },
    __wbg_String_8564e559799eccda: function (arg0, arg1) {
      const ret = String(arg1)
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
      const len1 = WASM_VECTOR_LEN
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true)
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true)
    },
    __wbg_Window_70131fc0c91e4b3c: function (arg0) {
      const ret = arg0.Window
      return ret
    },
    __wbg_WorkerGlobalScope_601c48015b8cc78e: function (arg0) {
      const ret = arg0.WorkerGlobalScope
      return ret
    },
    __wbg___wbindgen_boolean_get_fa956cfa2d1bd751: function (arg0) {
      const v = arg0
      const ret = typeof v === "boolean" ? v : undefined
      return isLikeNone(ret) ? 0xffffff : ret ? 1 : 0
    },
    __wbg___wbindgen_debug_string_c25d447a39f5578f: function (arg0, arg1) {
      const ret = debugString(arg1)
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
      const len1 = WASM_VECTOR_LEN
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true)
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true)
    },
    __wbg___wbindgen_in_aca499c5de7ff5e5: function (arg0, arg1) {
      const ret = arg0 in arg1
      return ret
    },
    __wbg___wbindgen_is_function_1ff95bcc5517c252: function (arg0) {
      const ret = typeof arg0 === "function"
      return ret
    },
    __wbg___wbindgen_is_null_ea9085d691f535d3: function (arg0) {
      const ret = arg0 === null
      return ret
    },
    __wbg___wbindgen_is_object_a27215656b807791: function (arg0) {
      const val = arg0
      const ret = typeof val === "object" && val !== null
      return ret
    },
    __wbg___wbindgen_is_string_ea5e6cc2e4141dfe: function (arg0) {
      const ret = typeof arg0 === "string"
      return ret
    },
    __wbg___wbindgen_is_undefined_c05833b95a3cf397: function (arg0) {
      const ret = arg0 === undefined
      return ret
    },
    __wbg___wbindgen_jsval_loose_eq_db4c3b15f63fc170: function (arg0, arg1) {
      const ret = arg0 == arg1
      return ret
    },
    __wbg___wbindgen_number_get_394265ed1e1b84ee: function (arg0, arg1) {
      const obj = arg1
      const ret = typeof obj === "number" ? obj : undefined
      getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true)
      getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true)
    },
    __wbg___wbindgen_string_get_b0ca35b86a603356: function (arg0, arg1) {
      const obj = arg1
      const ret = typeof obj === "string" ? obj : undefined
      var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
      var len1 = WASM_VECTOR_LEN
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true)
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true)
    },
    __wbg___wbindgen_throw_344f42d3211c4765: function (arg0, arg1) {
      throw new Error(getStringFromWasm0(arg0, arg1))
    },
    __wbg__wbg_cb_unref_fffb441def202758: function (arg0) {
      arg0._wbg_cb_unref()
    },
    __wbg_abort_8a4b90d8b05efcf8: function () {
      return handleError(function (arg0) {
        arg0.abort()
      }, arguments)
    },
    __wbg_addEventListener_520e749bbae24529: function () {
      return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3, arg4)
      }, arguments)
    },
    __wbg_addEventListener_d85450ee1320c989: function () {
      return handleError(function (arg0, arg1, arg2, arg3) {
        arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3)
      }, arguments)
    },
    __wbg_bound_696859c2d8dac8bf: function () {
      return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = IDBKeyRange.bound(arg0, arg1, arg2 !== 0, arg3 !== 0)
        return ret
      }, arguments)
    },
    __wbg_call_8a2dd23819f8a60a: function () {
      return handleError(function (arg0, arg1) {
        const ret = arg0.call(arg1)
        return ret
      }, arguments)
    },
    __wbg_call_a6e5c5dce5018821: function () {
      return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.call(arg1, arg2)
        return ret
      }, arguments)
    },
    __wbg_clearTimeout_3629d6209dfcc46e: function (arg0) {
      const ret = clearTimeout(arg0)
      return ret
    },
    __wbg_close_c65ca0257e895318: function () {
      return handleError(function (arg0) {
        arg0.close()
      }, arguments)
    },
    __wbg_code_1fc52b4142a112ac: function (arg0) {
      const ret = arg0.code
      return ret
    },
    __wbg_commit_e9c1332714c53826: function () {
      return handleError(function (arg0) {
        arg0.commit()
      }, arguments)
    },
    __wbg_createIndex_b4ed8919f11d086a: function () {
      return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.createIndex(getStringFromWasm0(arg1, arg2), arg3)
        return ret
      }, arguments)
    },
    __wbg_createObjectStore_ff668af6e79f0433: function () {
      return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.createObjectStore(getStringFromWasm0(arg1, arg2))
        return ret
      }, arguments)
    },
    __wbg_data_328de4280640da92: function (arg0) {
      const ret = arg0.data
      return ret
    },
    __wbg_debug_9475a59057a6d886: function (arg0, arg1) {
      var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice()
      wasm.__wbindgen_free(arg0, arg1 * 4, 4)
      console.debug(...v0)
    },
    __wbg_debug_eaef3b49d572d680: function (arg0, arg1) {
      var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice()
      wasm.__wbindgen_free(arg0, arg1 * 4, 4)
      console.debug(...v0)
    },
    __wbg_dispatchEvent_ca78eaf3d469bc25: function () {
      return handleError(function (arg0, arg1) {
        const ret = arg0.dispatchEvent(arg1)
        return ret
      }, arguments)
    },
    __wbg_done_89b2b13e91a60321: function (arg0) {
      const ret = arg0.done
      return ret
    },
    __wbg_entries_015dc610cd81ede0: function (arg0) {
      const ret = Object.entries(arg0)
      return ret
    },
    __wbg_error_71b0e71161a5f3a0: function (arg0, arg1) {
      var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice()
      wasm.__wbindgen_free(arg0, arg1 * 4, 4)
      console.error(...v0)
    },
    __wbg_error_a6fa202b58aa1cd3: function (arg0, arg1) {
      let deferred0_0
      let deferred0_1
      try {
        deferred0_0 = arg0
        deferred0_1 = arg1
        console.error(getStringFromWasm0(arg0, arg1))
      } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1)
      }
    },
    __wbg_error_ada576b138020f01: function (arg0, arg1) {
      var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice()
      wasm.__wbindgen_free(arg0, arg1 * 4, 4)
      console.error(...v0)
    },
    __wbg_error_becd7e1fe6ce0623: function () {
      return handleError(function (arg0) {
        const ret = arg0.error
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret)
      }, arguments)
    },
    __wbg_getAll_9da4db9daff27e28: function () {
      return handleError(function (arg0, arg1) {
        const ret = arg0.getAll(arg1)
        return ret
      }, arguments)
    },
    __wbg_get_507a50627bffa49b: function (arg0, arg1) {
      const ret = arg0[arg1 >>> 0]
      return ret
    },
    __wbg_get_78f252d074a84d0b: function () {
      return handleError(function (arg0, arg1) {
        const ret = Reflect.get(arg0, arg1)
        return ret
      }, arguments)
    },
    __wbg_get_c7eb1f358a7654df: function () {
      return handleError(function (arg0, arg1) {
        const ret = Reflect.get(arg0, arg1)
        return ret
      }, arguments)
    },
    __wbg_get_cefddcaffca4fbb7: function () {
      return handleError(function (arg0, arg1) {
        const ret = arg0.get(arg1)
        return ret
      }, arguments)
    },
    __wbg_get_unchecked_6e0ad6d2a41b06f6: function (arg0, arg1) {
      const ret = arg0[arg1 >>> 0]
      return ret
    },
    __wbg_get_with_ref_key_6412cf3094599694: function (arg0, arg1) {
      const ret = arg0[arg1]
      return ret
    },
    __wbg_global_e30ac0b7684506d0: function (arg0) {
      const ret = arg0.global
      return ret
    },
    __wbg_index_10b15a3a760a899d: function () {
      return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.index(getStringFromWasm0(arg1, arg2))
        return ret
      }, arguments)
    },
    __wbg_indexedDB_594b9e6820e78c00: function () {
      return handleError(function (arg0) {
        const ret = arg0.indexedDB
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret)
      }, arguments)
    },
    __wbg_indexedDB_a2139150e2ea2a08: function () {
      return handleError(function (arg0) {
        const ret = arg0.indexedDB
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret)
      }, arguments)
    },
    __wbg_indexedDB_c7dd741e3b661da5: function () {
      return handleError(function (arg0) {
        const ret = arg0.indexedDB
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret)
      }, arguments)
    },
    __wbg_instanceof_ArrayBuffer_4480b9e0068a8adb: function (arg0) {
      let result
      try {
        result = arg0 instanceof ArrayBuffer
      } catch (_) {
        result = false
      }
      const ret = result
      return ret
    },
    __wbg_instanceof_CursorSys_4b6a8aba0e823e75: function (arg0) {
      let result
      try {
        result = arg0 instanceof IDBCursorWithValue
      } catch (_) {
        result = false
      }
      const ret = result
      return ret
    },
    __wbg_instanceof_DomException_952faa8037702c00: function (arg0) {
      let result
      try {
        result = arg0 instanceof DOMException
      } catch (_) {
        result = false
      }
      const ret = result
      return ret
    },
    __wbg_instanceof_Error_1fdac9f13a8181ba: function (arg0) {
      let result
      try {
        result = arg0 instanceof Error
      } catch (_) {
        result = false
      }
      const ret = result
      return ret
    },
    __wbg_instanceof_IdbDatabase_1cc734ba1b040dd7: function (arg0) {
      let result
      try {
        result = arg0 instanceof IDBDatabase
      } catch (_) {
        result = false
      }
      const ret = result
      return ret
    },
    __wbg_instanceof_IdbRequest_fe7a4cb10800af5b: function (arg0) {
      let result
      try {
        result = arg0 instanceof IDBRequest
      } catch (_) {
        result = false
      }
      const ret = result
      return ret
    },
    __wbg_instanceof_Uint8Array_309b927aaf7a3fc7: function (arg0) {
      let result
      try {
        result = arg0 instanceof Uint8Array
      } catch (_) {
        result = false
      }
      const ret = result
      return ret
    },
    __wbg_isArray_0677c962b281d01a: function (arg0) {
      const ret = Array.isArray(arg0)
      return ret
    },
    __wbg_isSafeInteger_04f36e4056f1b851: function (arg0) {
      const ret = Number.isSafeInteger(arg0)
      return ret
    },
    __wbg_iterator_6f722e4a93058b71: function () {
      const ret = Symbol.iterator
      return ret
    },
    __wbg_length_1f0964f4a5e2c6d8: function (arg0) {
      const ret = arg0.length
      return ret
    },
    __wbg_length_370319915dc99107: function (arg0) {
      const ret = arg0.length
      return ret
    },
    __wbg_log_7a0760e115750083: function (arg0, arg1) {
      var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice()
      wasm.__wbindgen_free(arg0, arg1 * 4, 4)
      console.log(...v0)
    },
    __wbg_lowerBound_a00067ed8d582a72: function () {
      return handleError(function (arg0, arg1) {
        const ret = IDBKeyRange.lowerBound(arg0, arg1 !== 0)
        return ret
      }, arguments)
    },
    __wbg_message_8326fb1d549bebc5: function (arg0) {
      const ret = arg0.message
      return ret
    },
    __wbg_message_fb0e6e7854e6ea7a: function (arg0, arg1) {
      const ret = arg1.message
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
      const len1 = WASM_VECTOR_LEN
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true)
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true)
    },
    __wbg_name_9d2bcd24d4433cef: function (arg0, arg1) {
      const ret = arg1.name
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
      const len1 = WASM_VECTOR_LEN
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true)
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true)
    },
    __wbg_name_b0b4809690944614: function (arg0) {
      const ret = arg0.name
      return ret
    },
    __wbg_newVersion_faa932edc3b56859: function (arg0, arg1) {
      const ret = arg1.newVersion
      getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true)
      getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true)
    },
    __wbg_new_227d7c05414eb861: function () {
      const ret = new Error()
      return ret
    },
    __wbg_new_32b398fb48b6d94a: function () {
      const ret = new Array()
      return ret
    },
    __wbg_new_7796ffc7ed656783: function () {
      const ret = new Map()
      return ret
    },
    __wbg_new_b667d279fd5aa943: function (arg0, arg1) {
      const ret = new Error(getStringFromWasm0(arg0, arg1))
      return ret
    },
    __wbg_new_bf8729ffe10e9ee7: function () {
      return handleError(function (arg0, arg1) {
        const ret = new WebSocket(getStringFromWasm0(arg0, arg1))
        return ret
      }, arguments)
    },
    __wbg_new_cd45aabdf6073e84: function (arg0) {
      const ret = new Uint8Array(arg0)
      return ret
    },
    __wbg_new_da52cf8fe3429cb2: function () {
      const ret = new Object()
      return ret
    },
    __wbg_new_from_slice_77cdfb7977362f3c: function (arg0, arg1) {
      const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1))
      return ret
    },
    __wbg_new_typed_1824d93f294193e5: function (arg0, arg1) {
      try {
        var state0 = { a: arg0, b: arg1 }
        var cb0 = (arg0, arg1) => {
          const a = state0.a
          state0.a = 0
          try {
            return wasm_bindgen__convert__closures_____invoke__h55ccb1222fc95a46(a, state0.b, arg0, arg1)
          } finally {
            state0.a = a
          }
        }
        const ret = new Promise(cb0)
        return ret
      } finally {
        state0.a = 0
      }
    },
    __wbg_new_with_event_init_dict_7b62c0f9fb241877: function () {
      return handleError(function (arg0, arg1, arg2) {
        const ret = new CloseEvent(getStringFromWasm0(arg0, arg1), arg2)
        return ret
      }, arguments)
    },
    __wbg_new_with_u8_array_sequence_bdda73b0f202f149: function () {
      return handleError(function (arg0) {
        const ret = new Blob(arg0)
        return ret
      }, arguments)
    },
    __wbg_next_6dbf2c0ac8cde20f: function (arg0) {
      const ret = arg0.next
      return ret
    },
    __wbg_next_71f2aa1cb3d1e37e: function () {
      return handleError(function (arg0) {
        const ret = arg0.next()
        return ret
      }, arguments)
    },
    __wbg_now_86c0d4ba3fa605b8: function () {
      const ret = Date.now()
      return ret
    },
    __wbg_now_e7c6795a7f81e10f: function (arg0) {
      const ret = arg0.now()
      return ret
    },
    __wbg_objectStore_d5f47956b6c741e3: function () {
      return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.objectStore(getStringFromWasm0(arg1, arg2))
        return ret
      }, arguments)
    },
    __wbg_of_85f52f8b6491a7ca: function (arg0) {
      const ret = Array.of(arg0)
      return ret
    },
    __wbg_oldVersion_0fc5098aa13b3126: function (arg0) {
      const ret = arg0.oldVersion
      return ret
    },
    __wbg_open_8b445bd20535cb55: function () {
      return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.open(getStringFromWasm0(arg1, arg2))
        return ret
      }, arguments)
    },
    __wbg_parse_1c0d8a8656d7e016: function () {
      return handleError(function (arg0, arg1) {
        const ret = JSON.parse(getStringFromWasm0(arg0, arg1))
        return ret
      }, arguments)
    },
    __wbg_performance_3fcf6e32a7e1ed0a: function (arg0) {
      const ret = arg0.performance
      return ret
    },
    __wbg_prototypesetcall_4770620bbe4688a0: function (arg0, arg1, arg2) {
      Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2)
    },
    __wbg_push_d2ae3af0c1217ae6: function (arg0, arg1) {
      const ret = arg0.push(arg1)
      return ret
    },
    __wbg_put_a368805e3dcab3a7: function () {
      return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.put(arg1, arg2)
        return ret
      }, arguments)
    },
    __wbg_queueMicrotask_0ab5b2d2393e99b9: function (arg0) {
      const ret = arg0.queueMicrotask
      return ret
    },
    __wbg_queueMicrotask_6a09b7bc46549209: function (arg0) {
      queueMicrotask(arg0)
    },
    __wbg_readyState_50bc38c2a9e83db6: function (arg0) {
      const ret = arg0.readyState
      return ret
    },
    __wbg_readyState_9794af9795506c08: function (arg0) {
      const ret = arg0.readyState
      return (__wbindgen_enum_IdbRequestReadyState.indexOf(ret) + 1 || 3) - 1
    },
    __wbg_reason_5dc8e429d537d6a9: function (arg0, arg1) {
      const ret = arg1.reason
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
      const len1 = WASM_VECTOR_LEN
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true)
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true)
    },
    __wbg_removeEventListener_a3f23c70077bdcc1: function () {
      return handleError(function (arg0, arg1, arg2, arg3) {
        arg0.removeEventListener(getStringFromWasm0(arg1, arg2), arg3)
      }, arguments)
    },
    __wbg_resolve_2191a4dfe481c25b: function (arg0) {
      const ret = Promise.resolve(arg0)
      return ret
    },
    __wbg_result_2b1294a2bf8dc773: function () {
      return handleError(function (arg0) {
        const ret = arg0.result
        return ret
      }, arguments)
    },
    __wbg_send_1733c45567a373ff: function () {
      return handleError(function (arg0, arg1) {
        arg0.send(arg1)
      }, arguments)
    },
    __wbg_send_df98dd5ede9b3f4d: function () {
      return handleError(function (arg0, arg1, arg2) {
        arg0.send(getStringFromWasm0(arg1, arg2))
      }, arguments)
    },
    __wbg_setTimeout_56bcdccbad22fd44: function () {
      return handleError(function (arg0, arg1) {
        const ret = setTimeout(arg0, arg1)
        return ret
      }, arguments)
    },
    __wbg_set_575dd786d51585f8: function (arg0, arg1, arg2) {
      const ret = arg0.set(arg1, arg2)
      return ret
    },
    __wbg_set_6be42768c690e380: function (arg0, arg1, arg2) {
      arg0[arg1] = arg2
    },
    __wbg_set_8a16b38e4805b298: function (arg0, arg1, arg2) {
      arg0[arg1 >>> 0] = arg2
    },
    __wbg_set_binaryType_a37b086c78ca7c29: function (arg0, arg1) {
      arg0.binaryType = __wbindgen_enum_BinaryType[arg1]
    },
    __wbg_set_code_9d09aecd77d789f4: function (arg0, arg1) {
      arg0.code = arg1
    },
    __wbg_set_onabort_e8ad31807de2db24: function (arg0, arg1) {
      arg0.onabort = arg1
    },
    __wbg_set_once_51a9fb6b8af8a72b: function (arg0, arg1) {
      arg0.once = arg1 !== 0
    },
    __wbg_set_oncomplete_e6abb66d0ad42731: function (arg0, arg1) {
      arg0.oncomplete = arg1
    },
    __wbg_set_onerror_3488a474171ed56d: function (arg0, arg1) {
      arg0.onerror = arg1
    },
    __wbg_set_onerror_f8d31be44335c633: function (arg0, arg1) {
      arg0.onerror = arg1
    },
    __wbg_set_onsuccess_cd0c3642a2873e66: function (arg0, arg1) {
      arg0.onsuccess = arg1
    },
    __wbg_set_onupgradeneeded_7b2cf4ba1c57e655: function (arg0, arg1) {
      arg0.onupgradeneeded = arg1
    },
    __wbg_set_reason_18dc06ea0d60243b: function (arg0, arg1, arg2) {
      arg0.reason = getStringFromWasm0(arg1, arg2)
    },
    __wbg_stack_3b0d974bbf31e44f: function (arg0, arg1) {
      const ret = arg1.stack
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc)
      const len1 = WASM_VECTOR_LEN
      getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true)
      getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true)
    },
    __wbg_static_accessor_GLOBAL_4ef717fb391d88b7: function () {
      const ret = typeof global === "undefined" ? null : global
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret)
    },
    __wbg_static_accessor_GLOBAL_THIS_8d1badc68b5a74f4: function () {
      const ret = typeof globalThis === "undefined" ? null : globalThis
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret)
    },
    __wbg_static_accessor_SELF_146583524fe1469b: function () {
      const ret = typeof self === "undefined" ? null : self
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret)
    },
    __wbg_static_accessor_WINDOW_f2829a2234d7819e: function () {
      const ret = typeof window === "undefined" ? null : window
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret)
    },
    __wbg_target_e759594a8d965ed7: function (arg0) {
      const ret = arg0.target
      return isLikeNone(ret) ? 0 : addToExternrefTable0(ret)
    },
    __wbg_then_6ec10ae38b3e92f7: function (arg0, arg1) {
      const ret = arg0.then(arg1)
      return ret
    },
    __wbg_toString_b201c2690bbe445a: function (arg0) {
      const ret = arg0.toString()
      return ret
    },
    __wbg_toString_bac9199ff382784d: function (arg0) {
      const ret = arg0.toString()
      return ret
    },
    __wbg_transaction_a00de84491e23887: function () {
      return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.transaction(getStringFromWasm0(arg1, arg2))
        return ret
      }, arguments)
    },
    __wbg_transaction_d911d96b4b0af154: function () {
      return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.transaction(getStringFromWasm0(arg1, arg2), __wbindgen_enum_IdbTransactionMode[arg3])
        return ret
      }, arguments)
    },
    __wbg_transaction_d93cc0feb20672da: function (arg0) {
      const ret = arg0.transaction
      return ret
    },
    __wbg_upperBound_68f285e0e589eb0e: function () {
      return handleError(function (arg0, arg1) {
        const ret = IDBKeyRange.upperBound(arg0, arg1 !== 0)
        return ret
      }, arguments)
    },
    __wbg_value_a5d5488a9589444a: function (arg0) {
      const ret = arg0.value
      return ret
    },
    __wbg_warn_3a37cdd7216f1479: function (arg0, arg1) {
      var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice()
      wasm.__wbindgen_free(arg0, arg1 * 4, 4)
      console.warn(...v0)
    },
    __wbg_wasClean_3c7aa2335da09e74: function (arg0) {
      const ret = arg0.wasClean
      return ret
    },
    __wbindgen_cast_0000000000000001: function (arg0, arg1) {
      // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 1059, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h69af921b525aa026)
      return ret
    },
    __wbindgen_cast_0000000000000002: function (arg0, arg1) {
      // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("CloseEvent")], shim_idx: 775, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579)
      return ret
    },
    __wbindgen_cast_0000000000000003: function (arg0, arg1) {
      // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 677, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h04b81f5efcb0df6f)
      return ret
    },
    __wbindgen_cast_0000000000000004: function (arg0, arg1) {
      // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 775, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579_3)
      return ret
    },
    __wbindgen_cast_0000000000000005: function (arg0, arg1) {
      // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("IDBVersionChangeEvent")], shim_idx: 634, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hfc4f4eb54d361465)
      return ret
    },
    __wbindgen_cast_0000000000000006: function (arg0, arg1) {
      // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("MessageEvent")], shim_idx: 775, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579_5)
      return ret
    },
    __wbindgen_cast_0000000000000007: function (arg0, arg1) {
      // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 675, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__he561c48a3e9220eb)
      return ret
    },
    __wbindgen_cast_0000000000000008: function (arg0, arg1) {
      // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 777, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hd7c0f5330375ec49)
      return ret
    },
    __wbindgen_cast_0000000000000009: function (arg0, arg1) {
      // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 936, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
      const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h84f8f095dc5775a9)
      return ret
    },
    __wbindgen_cast_000000000000000a: function (arg0) {
      // Cast intrinsic for `F64 -> Externref`.
      const ret = arg0
      return ret
    },
    __wbindgen_cast_000000000000000b: function (arg0, arg1) {
      // Cast intrinsic for `Ref(String) -> Externref`.
      const ret = getStringFromWasm0(arg0, arg1)
      return ret
    },
    __wbindgen_cast_000000000000000c: function (arg0, arg1) {
      var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice()
      wasm.__wbindgen_free(arg0, arg1 * 4, 4)
      // Cast intrinsic for `Vector(NamedExternref("ChannelInfo")) -> Externref`.
      const ret = v0
      return ret
    },
    __wbindgen_cast_000000000000000d: function (arg0, arg1) {
      var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice()
      wasm.__wbindgen_free(arg0, arg1 * 4, 4)
      // Cast intrinsic for `Vector(NamedExternref("Server")) -> Externref`.
      const ret = v0
      return ret
    },
    __wbindgen_init_externref_table: function () {
      const table = wasm.__wbindgen_externrefs
      const offset = table.grow(4)
      table.set(0, undefined)
      table.set(offset + 0, undefined)
      table.set(offset + 1, null)
      table.set(offset + 2, true)
      table.set(offset + 3, false)
    },
  }
  return {
    __proto__: null,
    "./core_wasm_bg.js": import0,
  }
}

function wasm_bindgen__convert__closures_____invoke__he561c48a3e9220eb(arg0, arg1) {
  wasm.wasm_bindgen__convert__closures_____invoke__he561c48a3e9220eb(arg0, arg1)
}

function wasm_bindgen__convert__closures_____invoke__hd7c0f5330375ec49(arg0, arg1) {
  wasm.wasm_bindgen__convert__closures_____invoke__hd7c0f5330375ec49(arg0, arg1)
}

function wasm_bindgen__convert__closures_____invoke__h84f8f095dc5775a9(arg0, arg1) {
  wasm.wasm_bindgen__convert__closures_____invoke__h84f8f095dc5775a9(arg0, arg1)
}

function wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579(arg0, arg1, arg2) {
  wasm.wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579(arg0, arg1, arg2)
}

function wasm_bindgen__convert__closures_____invoke__h04b81f5efcb0df6f(arg0, arg1, arg2) {
  wasm.wasm_bindgen__convert__closures_____invoke__h04b81f5efcb0df6f(arg0, arg1, arg2)
}

function wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579_3(arg0, arg1, arg2) {
  wasm.wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579_3(arg0, arg1, arg2)
}

function wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579_5(arg0, arg1, arg2) {
  wasm.wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579_5(arg0, arg1, arg2)
}

function wasm_bindgen__convert__closures_____invoke__h69af921b525aa026(arg0, arg1, arg2) {
  const ret = wasm.wasm_bindgen__convert__closures_____invoke__h69af921b525aa026(arg0, arg1, arg2)
  if (ret[1]) {
    throw takeFromExternrefTable0(ret[0])
  }
}

function wasm_bindgen__convert__closures_____invoke__hfc4f4eb54d361465(arg0, arg1, arg2) {
  const ret = wasm.wasm_bindgen__convert__closures_____invoke__hfc4f4eb54d361465(arg0, arg1, arg2)
  if (ret[1]) {
    throw takeFromExternrefTable0(ret[0])
  }
}

function wasm_bindgen__convert__closures_____invoke__h55ccb1222fc95a46(arg0, arg1, arg2, arg3) {
  wasm.wasm_bindgen__convert__closures_____invoke__h55ccb1222fc95a46(arg0, arg1, arg2, arg3)
}

const __wbindgen_enum_BinaryType = ["blob", "arraybuffer"]

const __wbindgen_enum_IdbRequestReadyState = ["pending", "done"]

const __wbindgen_enum_IdbTransactionMode = ["readonly", "readwrite", "versionchange", "readwriteflush", "cleanup"]

function addToExternrefTable0(obj) {
  const idx = wasm.__externref_table_alloc()
  wasm.__wbindgen_externrefs.set(idx, obj)
  return idx
}

const CLOSURE_DTORS = typeof FinalizationRegistry === "undefined" ? { register: () => {}, unregister: () => {} } : new FinalizationRegistry((state) => wasm.__wbindgen_destroy_closure(state.a, state.b))

function debugString(val) {
  // primitive types
  const type = typeof val
  if (type == "number" || type == "boolean" || val == null) {
    return `${val}`
  }
  if (type == "string") {
    return `"${val}"`
  }
  if (type == "symbol") {
    const description = val.description
    if (description == null) {
      return "Symbol"
    } else {
      return `Symbol(${description})`
    }
  }
  if (type == "function") {
    const name = val.name
    if (typeof name == "string" && name.length > 0) {
      return `Function(${name})`
    } else {
      return "Function"
    }
  }
  // objects
  if (Array.isArray(val)) {
    const length = val.length
    let debug = "["
    if (length > 0) {
      debug += debugString(val[0])
    }
    for (let i = 1; i < length; i++) {
      debug += ", " + debugString(val[i])
    }
    debug += "]"
    return debug
  }
  // Test for built-in
  const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val))
  let className
  if (builtInMatches && builtInMatches.length > 1) {
    className = builtInMatches[1]
  } else {
    // Failed to match the standard '[object ClassName]'
    return toString.call(val)
  }
  if (className == "Object") {
    // we're a user defined class or Object
    // JSON.stringify avoids problems with cycles, and is generally much
    // easier than looping through ownProperties of `val`.
    try {
      return "Object(" + JSON.stringify(val) + ")"
    } catch (_) {
      return "Object"
    }
  }
  // errors
  if (val instanceof Error) {
    return `${val.name}: ${val.message}\n${val.stack}`
  }
  // TODO we could test for more things here, like `Set`s and `Map`s.
  return className
}

function getArrayJsValueFromWasm0(ptr, len) {
  ptr = ptr >>> 0
  const mem = getDataViewMemory0()
  const result = []
  for (let i = ptr; i < ptr + 4 * len; i += 4) {
    result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)))
  }
  wasm.__externref_drop_slice(ptr, len)
  return result
}

function getArrayU8FromWasm0(ptr, len) {
  ptr = ptr >>> 0
  return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len)
}

let cachedDataViewMemory0 = null
function getDataViewMemory0() {
  if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
    cachedDataViewMemory0 = new DataView(wasm.memory.buffer)
  }
  return cachedDataViewMemory0
}

function getStringFromWasm0(ptr, len) {
  return decodeText(ptr >>> 0, len)
}

let cachedUint8ArrayMemory0 = null
function getUint8ArrayMemory0() {
  if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
    cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer)
  }
  return cachedUint8ArrayMemory0
}

function handleError(f, args) {
  try {
    return f.apply(this, args)
  } catch (e) {
    const idx = addToExternrefTable0(e)
    wasm.__wbindgen_exn_store(idx)
  }
}

function isLikeNone(x) {
  return x === undefined || x === null
}

function makeMutClosure(arg0, arg1, f) {
  const state = { a: arg0, b: arg1, cnt: 1 }
  const real = (...args) => {
    // First up with a closure we increment the internal reference
    // count. This ensures that the Rust closure environment won't
    // be deallocated while we're invoking it.
    state.cnt++
    const a = state.a
    state.a = 0
    try {
      return f(a, state.b, ...args)
    } finally {
      state.a = a
      real._wbg_cb_unref()
    }
  }
  real._wbg_cb_unref = () => {
    if (--state.cnt === 0) {
      wasm.__wbindgen_destroy_closure(state.a, state.b)
      state.a = 0
      CLOSURE_DTORS.unregister(state)
    }
  }
  CLOSURE_DTORS.register(real, state, state)
  return real
}

function passStringToWasm0(arg, malloc, realloc) {
  if (realloc === undefined) {
    const buf = cachedTextEncoder.encode(arg)
    const ptr = malloc(buf.length, 1) >>> 0
    getUint8ArrayMemory0()
      .subarray(ptr, ptr + buf.length)
      .set(buf)
    WASM_VECTOR_LEN = buf.length
    return ptr
  }

  let len = arg.length
  let ptr = malloc(len, 1) >>> 0

  const mem = getUint8ArrayMemory0()

  let offset = 0

  for (; offset < len; offset++) {
    const code = arg.charCodeAt(offset)
    if (code > 0x7f) break
    mem[ptr + offset] = code
  }
  if (offset !== len) {
    if (offset !== 0) {
      arg = arg.slice(offset)
    }
    ptr = realloc(ptr, len, (len = offset + arg.length * 3), 1) >>> 0
    const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len)
    const ret = cachedTextEncoder.encodeInto(arg, view)

    offset += ret.written
    ptr = realloc(ptr, len, offset, 1) >>> 0
  }

  WASM_VECTOR_LEN = offset
  return ptr
}

function takeFromExternrefTable0(idx) {
  const value = wasm.__wbindgen_externrefs.get(idx)
  wasm.__externref_table_dealloc(idx)
  return value
}

let cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true })
cachedTextDecoder.decode()
const MAX_SAFARI_DECODE_BYTES = 2146435072
let numBytesDecoded = 0
function decodeText(ptr, len) {
  numBytesDecoded += len
  if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
    cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true })
    cachedTextDecoder.decode()
    numBytesDecoded = len
  }
  return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len))
}

const cachedTextEncoder = new TextEncoder()

if (!("encodeInto" in cachedTextEncoder)) {
  cachedTextEncoder.encodeInto = function (arg, view) {
    const buf = cachedTextEncoder.encode(arg)
    view.set(buf)
    return {
      read: arg.length,
      written: buf.length,
    }
  }
}

let WASM_VECTOR_LEN = 0

let wasmModule, wasmInstance, wasm
function __wbg_finalize_init(instance, module) {
  wasmInstance = instance
  wasm = instance.exports
  wasmModule = module
  cachedDataViewMemory0 = null
  cachedUint8ArrayMemory0 = null
  wasm.__wbindgen_start()
  return wasm
}

async function __wbg_load(module, imports) {
  if (typeof Response === "function" && module instanceof Response) {
    if (typeof WebAssembly.instantiateStreaming === "function") {
      try {
        return await WebAssembly.instantiateStreaming(module, imports)
      } catch (e) {
        const validResponse = module.ok && expectedResponseType(module.type)

        if (validResponse && module.headers.get("Content-Type") !== "application/wasm") {
          console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e)
        } else {
          throw e
        }
      }
    }

    const bytes = await module.arrayBuffer()
    return await WebAssembly.instantiate(bytes, imports)
  } else {
    const instance = await WebAssembly.instantiate(module, imports)

    if (instance instanceof WebAssembly.Instance) {
      return { instance, module }
    } else {
      return instance
    }
  }

  function expectedResponseType(type) {
    switch (type) {
      case "basic":
      case "cors":
      case "default":
        return true
    }
    return false
  }
}

function initSync(module) {
  if (wasm !== undefined) return wasm

  if (module !== undefined) {
    if (Object.getPrototypeOf(module) === Object.prototype) {
      ;({ module } = module)
    } else {
      console.warn("using deprecated parameters for `initSync()`; pass a single object instead")
    }
  }

  const imports = __wbg_get_imports()
  if (!(module instanceof WebAssembly.Module)) {
    module = new WebAssembly.Module(module)
  }
  const instance = new WebAssembly.Instance(module, imports)
  return __wbg_finalize_init(instance, module)
}

async function __wbg_init(module_or_path) {
  if (wasm !== undefined) return wasm

  if (module_or_path !== undefined) {
    if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
      ;({ module_or_path } = module_or_path)
    } else {
      console.warn("using deprecated parameters for the initialization function; pass a single object instead")
    }
  }

  if (module_or_path === undefined) {
    module_or_path = new URL("core_wasm_bg.wasm", import.meta.url)
  }
  const imports = __wbg_get_imports()

  if (typeof module_or_path === "string" || (typeof Request === "function" && module_or_path instanceof Request) || (typeof URL === "function" && module_or_path instanceof URL)) {
    module_or_path = fetch(module_or_path)
  }

  const { instance, module } = await __wbg_load(await module_or_path, imports)

  return __wbg_finalize_init(instance, module)
}

export { initSync, __wbg_init as default }
