let Y=0,T=null,V=`undefined`,a0=`boolean`,R=128,a1=`string`,U=1,a2=`Object`,W=`utf-8`,$=`number`,a4=4,_=`function`,a6=267,Q=Array,X=Error,a3=FinalizationRegistry,a5=Object,Z=Uint8Array,S=undefined;var u=(a=>{const b=typeof a;if(b==$||b==a0||a==T){return `${a}`};if(b==a1){return `"${a}"`};if(b==`symbol`){const b=a.description;if(b==T){return `Symbol`}else{return `Symbol(${b})`}};if(b==_){const b=a.name;if(typeof b==a1&&b.length>Y){return `Function(${b})`}else{return `Function`}};if(Q.isArray(a)){const b=a.length;let c=`[`;if(b>Y){c+=u(a[Y])};for(let d=U;d<b;d++){c+=`, `+ u(a[d])};c+=`]`;return c};const c=/\[object ([^\]]+)\]/.exec(toString.call(a));let d;if(c.length>U){d=c[U]}else{return toString.call(a)};if(d==a2){try{return `Object(`+ JSON.stringify(a)+ `)`}catch(a){return a2}};if(a instanceof X){return `${a.name}: ${a.message}\n${a.stack}`};return d});var M=((a,b)=>{});var y=((b,c)=>{a._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hfab3d848ad6c0f5c(b,c)});function I(b,c){try{return b.apply(this,c)}catch(b){a.__wbindgen_exn_store(g(b))}}var p=(a=>a===S||a===T);var c=(a=>b[a]);var D=((b,c,d)=>{a._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h2c467dcbb13db7be(b,c,g(d))});var L=(()=>{const b={};b.wbg={};b.wbg.__wbindgen_object_drop_ref=(a=>{f(a)});b.wbg.__wbindgen_cb_drop=(a=>{const b=f(a).original;if(b.cnt--==U){b.a=Y;return !0};const c=!1;return c});b.wbg.__wbindgen_object_clone_ref=(a=>{const b=c(a);return g(b)});b.wbg.__wbindgen_string_new=((a,b)=>{const c=k(a,b);return g(c)});b.wbg.__wbindgen_is_string=(a=>{const b=typeof c(a)===a1;return b});b.wbg.__wbindgen_string_get=((b,d)=>{const e=c(d);const f=typeof e===a1?e:S;var g=p(f)?Y:o(f,a.__wbindgen_malloc,a.__wbindgen_realloc);var h=l;r()[b/a4+ U]=h;r()[b/a4+ Y]=g});b.wbg.__wbg_listenerid_6dcf1c62b7b7de58=((a,b)=>{const d=c(b).__yew_listener_id;r()[a/a4+ U]=p(d)?Y:d;r()[a/a4+ Y]=!p(d)});b.wbg.__wbg_setlistenerid_f2e783343fa0cec1=((a,b)=>{c(a).__yew_listener_id=b>>>Y});b.wbg.__wbg_setsubtreeid_e1fab6b578c800cf=((a,b)=>{c(a).__yew_subtree_id=b>>>Y});b.wbg.__wbg_subtreeid_e80a1798fee782f9=((a,b)=>{const d=c(b).__yew_subtree_id;r()[a/a4+ U]=p(d)?Y:d;r()[a/a4+ Y]=!p(d)});b.wbg.__wbg_cachekey_b81c1aacc6a0645c=((a,b)=>{const d=c(b).__yew_subtree_cache_key;r()[a/a4+ U]=p(d)?Y:d;r()[a/a4+ Y]=!p(d)});b.wbg.__wbg_setcachekey_75bcd45312087529=((a,b)=>{c(a).__yew_subtree_cache_key=b>>>Y});b.wbg.__wbg_new_abda76e883ba8a5f=(()=>{const a=new X();return g(a)});b.wbg.__wbg_stack_658279fe44541cf6=((b,d)=>{const e=c(d).stack;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbg_error_f851667af71bcfc6=((b,c)=>{let d;let e;try{d=b;e=c;console.error(k(b,c))}finally{a.__wbindgen_free(d,e,U)}});b.wbg.__wbg_queueMicrotask_481971b0d87f3dd4=(a=>{queueMicrotask(c(a))});b.wbg.__wbg_queueMicrotask_3cbae2ec6b6cd3d6=(a=>{const b=c(a).queueMicrotask;return g(b)});b.wbg.__wbindgen_is_function=(a=>{const b=typeof c(a)===_;return b});b.wbg.__wbindgen_is_object=(a=>{const b=c(a);const d=typeof b===`object`&&b!==T;return d});b.wbg.__wbindgen_is_undefined=(a=>{const b=c(a)===S;return b});b.wbg.__wbindgen_in=((a,b)=>{const d=c(a) in c(b);return d});b.wbg.__wbindgen_error_new=((a,b)=>{const c=new X(k(a,b));return g(c)});b.wbg.__wbindgen_jsval_loose_eq=((a,b)=>{const d=c(a)==c(b);return d});b.wbg.__wbindgen_boolean_get=(a=>{const b=c(a);const d=typeof b===a0?(b?U:Y):2;return d});b.wbg.__wbindgen_number_get=((a,b)=>{const d=c(b);const e=typeof d===$?d:S;t()[a/8+ U]=p(e)?Y:e;r()[a/a4+ Y]=!p(e)});b.wbg.__wbindgen_as_number=(a=>{const b=+c(a);return b});b.wbg.__wbindgen_number_new=(a=>{const b=a;return g(b)});b.wbg.__wbg_getwithrefkey_edc2c8960f0f1191=((a,b)=>{const d=c(a)[c(b)];return g(d)});b.wbg.__wbg_error_a526fb08a0205972=((b,c)=>{var d=H(b,c).slice();a.__wbindgen_free(b,c*a4,a4);console.error(...d)});b.wbg.__wbg_body_edb1908d3ceff3a1=(a=>{const b=c(a).body;return p(b)?Y:g(b)});b.wbg.__wbg_createElement_8bae7856a4bb7411=function(){return I(((a,b,d)=>{const e=c(a).createElement(k(b,d));return g(e)}),arguments)};b.wbg.__wbg_createElementNS_556a62fb298be5a2=function(){return I(((a,b,d,e,f)=>{const h=c(a).createElementNS(b===Y?S:k(b,d),k(e,f));return g(h)}),arguments)};b.wbg.__wbg_createTextNode_0c38fd80a5b2284d=((a,b,d)=>{const e=c(a).createTextNode(k(b,d));return g(e)});b.wbg.__wbg_querySelector_a5f74efc5fa193dd=function(){return I(((a,b,d)=>{const e=c(a).querySelector(k(b,d));return p(e)?Y:g(e)}),arguments)};b.wbg.__wbg_instanceof_Window_f401953a2cf86220=(a=>{let b;try{b=c(a) instanceof Window}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_document_5100775d18896c16=(a=>{const b=c(a).document;return p(b)?Y:g(b)});b.wbg.__wbg_location_2951b5ee34f19221=(a=>{const b=c(a).location;return g(b)});b.wbg.__wbg_history_bc4057de66a2015f=function(){return I((a=>{const b=c(a).history;return g(b)}),arguments)};b.wbg.__wbg_instanceof_Element_6945fc210db80ea9=(a=>{let b;try{b=c(a) instanceof Element}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_namespaceURI_5235ee79fd5f6781=((b,d)=>{const e=c(d).namespaceURI;var f=p(e)?Y:o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);var g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbg_setinnerHTML_26d69b59e1af99c7=((a,b,d)=>{c(a).innerHTML=k(b,d)});b.wbg.__wbg_outerHTML_e073aa84e7bc1eaf=((b,d)=>{const e=c(d).outerHTML;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbg_removeAttribute_1b10a06ae98ebbd1=function(){return I(((a,b,d)=>{c(a).removeAttribute(k(b,d))}),arguments)};b.wbg.__wbg_setAttribute_3c9f6c303b696daa=function(){return I(((a,b,d,e,f)=>{c(a).setAttribute(k(b,d),k(e,f))}),arguments)};b.wbg.__wbg_wasClean_8222e9acf5c5ad07=(a=>{const b=c(a).wasClean;return b});b.wbg.__wbg_code_5ee5dcc2842228cd=(a=>{const b=c(a).code;return b});b.wbg.__wbg_reason_5ed6709323849cb1=((b,d)=>{const e=c(d).reason;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbg_newwitheventinitdict_c939a6b964db4d91=function(){return I(((a,b,d)=>{const e=new CloseEvent(k(a,b),c(d));return g(e)}),arguments)};b.wbg.__wbg_href_2edbae9e92cdfeff=((b,d)=>{const e=c(d).href;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbg_parentNode_6be3abff20e1a5fb=(a=>{const b=c(a).parentNode;return p(b)?Y:g(b)});b.wbg.__wbg_parentElement_347524db59fc2976=(a=>{const b=c(a).parentElement;return p(b)?Y:g(b)});b.wbg.__wbg_childNodes_118168e8b23bcb9b=(a=>{const b=c(a).childNodes;return g(b)});b.wbg.__wbg_lastChild_83efe6d5da370e1f=(a=>{const b=c(a).lastChild;return p(b)?Y:g(b)});b.wbg.__wbg_nextSibling_709614fdb0fb7a66=(a=>{const b=c(a).nextSibling;return p(b)?Y:g(b)});b.wbg.__wbg_setnodeValue_94b86af0cda24b90=((a,b,d)=>{c(a).nodeValue=b===Y?S:k(b,d)});b.wbg.__wbg_textContent_8e392d624539e731=((b,d)=>{const e=c(d).textContent;var f=p(e)?Y:o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);var g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbg_cloneNode_e19c313ea20d5d1d=function(){return I((a=>{const b=c(a).cloneNode();return g(b)}),arguments)};b.wbg.__wbg_insertBefore_d2a001abf538c1f8=function(){return I(((a,b,d)=>{const e=c(a).insertBefore(c(b),c(d));return g(e)}),arguments)};b.wbg.__wbg_removeChild_96bbfefd2f5a0261=function(){return I(((a,b)=>{const d=c(a).removeChild(c(b));return g(d)}),arguments)};b.wbg.__wbg_addEventListener_53b787075bd5e003=function(){return I(((a,b,d,e)=>{c(a).addEventListener(k(b,d),c(e))}),arguments)};b.wbg.__wbg_addEventListener_4283b15b4f039eb5=function(){return I(((a,b,d,e,f)=>{c(a).addEventListener(k(b,d),c(e),c(f))}),arguments)};b.wbg.__wbg_dispatchEvent_63c0c01600a98fd2=function(){return I(((a,b)=>{const d=c(a).dispatchEvent(c(b));return d}),arguments)};b.wbg.__wbg_removeEventListener_92cb9b3943463338=function(){return I(((a,b,d,e)=>{c(a).removeEventListener(k(b,d),c(e))}),arguments)};b.wbg.__wbg_removeEventListener_5d31483804421bfa=function(){return I(((a,b,d,e,f)=>{c(a).removeEventListener(k(b,d),c(e),f!==Y)}),arguments)};b.wbg.__wbg_setchecked_931ff2ed2cd3ebfd=((a,b)=>{c(a).checked=b!==Y});b.wbg.__wbg_value_47fe6384562f52ab=((b,d)=>{const e=c(d).value;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbg_setvalue_78cb4f1fef58ae98=((a,b,d)=>{c(a).value=k(b,d)});b.wbg.__wbg_debug_7d879afce6cf56cb=((a,b,d,e)=>{console.debug(c(a),c(b),c(d),c(e))});b.wbg.__wbg_error_8e3928cfb8a43e2b=(a=>{console.error(c(a))});b.wbg.__wbg_error_696630710900ec44=((a,b,d,e)=>{console.error(c(a),c(b),c(d),c(e))});b.wbg.__wbg_info_80803d9a3f0aad16=((a,b,d,e)=>{console.info(c(a),c(b),c(d),c(e))});b.wbg.__wbg_log_151eb4333ef0fe39=((a,b,d,e)=>{console.log(c(a),c(b),c(d),c(e))});b.wbg.__wbg_warn_5d3f783b0bae8943=((a,b,d,e)=>{console.warn(c(a),c(b),c(d),c(e))});b.wbg.__wbg_bubbles_abce839854481bc6=(a=>{const b=c(a).bubbles;return b});b.wbg.__wbg_cancelBubble_c0aa3172524eb03c=(a=>{const b=c(a).cancelBubble;return b});b.wbg.__wbg_composedPath_58473fd5ae55f2cd=(a=>{const b=c(a).composedPath();return g(b)});b.wbg.__wbg_data_3ce7c145ca4fbcdc=(a=>{const b=c(a).data;return g(b)});b.wbg.__wbg_state_9cc3f933b7d50acb=function(){return I((a=>{const b=c(a).state;return g(b)}),arguments)};b.wbg.__wbg_instanceof_ShadowRoot_9db040264422e84a=(a=>{let b;try{b=c(a) instanceof ShadowRoot}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_host_c667c7623404d6bf=(a=>{const b=c(a).host;return g(b)});b.wbg.__wbg_href_706b235ecfe6848c=function(){return I(((b,d)=>{const e=c(d).href;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f}),arguments)};b.wbg.__wbg_hostname_3d9f22c60dc5bec6=function(){return I(((b,d)=>{const e=c(d).hostname;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f}),arguments)};b.wbg.__wbg_port_b8d9a9c4e2b26efa=function(){return I(((b,d)=>{const e=c(d).port;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f}),arguments)};b.wbg.__wbg_pathname_5449afe3829f96a1=function(){return I(((b,d)=>{const e=c(d).pathname;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f}),arguments)};b.wbg.__wbg_search_489f12953342ec1f=function(){return I(((b,d)=>{const e=c(d).search;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f}),arguments)};b.wbg.__wbg_hash_553098e838e06c1d=function(){return I(((b,d)=>{const e=c(d).hash;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f}),arguments)};b.wbg.__wbg_readyState_1c157e4ea17c134a=(a=>{const b=c(a).readyState;return b});b.wbg.__wbg_setbinaryType_b0cf5103cd561959=((a,b)=>{c(a).binaryType=f(b)});b.wbg.__wbg_new_6c74223c77cfabad=function(){return I(((a,b)=>{const c=new WebSocket(k(a,b));return g(c)}),arguments)};b.wbg.__wbg_close_acd9532ff5c093ea=function(){return I((a=>{c(a).close()}),arguments)};b.wbg.__wbg_send_70603dff16b81b66=function(){return I(((a,b,d)=>{c(a).send(k(b,d))}),arguments)};b.wbg.__wbg_send_5fcd7bab9777194e=function(){return I(((a,b,d)=>{c(a).send(J(b,d))}),arguments)};b.wbg.__wbg_pathname_c5fe403ef9525ec6=((b,d)=>{const e=c(d).pathname;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbg_search_c68f506c44be6d1e=((b,d)=>{const e=c(d).search;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbg_hash_cdea7a9b7e684a42=((b,d)=>{const e=c(d).hash;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbg_new_67853c351755d2cf=function(){return I(((a,b)=>{const c=new URL(k(a,b));return g(c)}),arguments)};b.wbg.__wbg_newwithbase_6aabbfb1b2e6a1cb=function(){return I(((a,b,c,d)=>{const e=new URL(k(a,b),k(c,d));return g(e)}),arguments)};b.wbg.__wbg_value_d7f5bfbd9302c14b=((b,d)=>{const e=c(d).value;const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbg_setvalue_090972231f0a4f6f=((a,b,d)=>{c(a).value=k(b,d)});b.wbg.__wbg_get_bd8e338fbd5f5cc8=((a,b)=>{const d=c(a)[b>>>Y];return g(d)});b.wbg.__wbg_length_cd7af8117672b8b8=(a=>{const b=c(a).length;return b});b.wbg.__wbg_newnoargs_e258087cd0daa0ea=((a,b)=>{const c=new Function(k(a,b));return g(c)});b.wbg.__wbg_call_27c0f87801dedf93=function(){return I(((a,b)=>{const d=c(a).call(c(b));return g(d)}),arguments)};b.wbg.__wbg_new_72fb9a18b5ae2624=(()=>{const a=new a5();return g(a)});b.wbg.__wbg_self_ce0dbfc45cf2f5be=function(){return I((()=>{const a=self.self;return g(a)}),arguments)};b.wbg.__wbg_window_c6fb939a7f436783=function(){return I((()=>{const a=window.window;return g(a)}),arguments)};b.wbg.__wbg_globalThis_d1e6af4856ba331b=function(){return I((()=>{const a=globalThis.globalThis;return g(a)}),arguments)};b.wbg.__wbg_global_207b558942527489=function(){return I((()=>{const a=global.global;return g(a)}),arguments)};b.wbg.__wbg_from_89e3fc3ba5e6fb48=(a=>{const b=Q.from(c(a));return g(b)});b.wbg.__wbg_instanceof_ArrayBuffer_836825be07d4c9d2=(a=>{let b;try{b=c(a) instanceof ArrayBuffer}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_instanceof_Error_e20bb56fd5591a93=(a=>{let b;try{b=c(a) instanceof X}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_message_5bf28016c2b49cfb=(a=>{const b=c(a).message;return g(b)});b.wbg.__wbg_name_e7429f0dda6079e2=(a=>{const b=c(a).name;return g(b)});b.wbg.__wbg_toString_ffe4c9ea3b3532e9=(a=>{const b=c(a).toString();return g(b)});b.wbg.__wbg_isSafeInteger_f7b04ef02296c4d2=(a=>{const b=Number.isSafeInteger(c(a));return b});b.wbg.__wbg_entries_95cc2c823b285a09=(a=>{const b=a5.entries(c(a));return g(b)});b.wbg.__wbg_is_010fdc0f4ab96916=((a,b)=>{const d=a5.is(c(a),c(b));return d});b.wbg.__wbg_resolve_b0083a7967828ec8=(a=>{const b=Promise.resolve(c(a));return g(b)});b.wbg.__wbg_then_0c86a60e8fcfe9f6=((a,b)=>{const d=c(a).then(c(b));return g(d)});b.wbg.__wbg_buffer_12d079cc21e14bdb=(a=>{const b=c(a).buffer;return g(b)});b.wbg.__wbg_new_63b92bc8671ed464=(a=>{const b=new Z(c(a));return g(b)});b.wbg.__wbg_set_a47bac70306a19a7=((a,b,d)=>{c(a).set(c(b),d>>>Y)});b.wbg.__wbg_length_c20a40f15020d68a=(a=>{const b=c(a).length;return b});b.wbg.__wbg_instanceof_Uint8Array_2b3bbecd033d19f6=(a=>{let b;try{b=c(a) instanceof Z}catch(a){b=!1}const d=b;return d});b.wbg.__wbg_set_1f9b04f170055d33=function(){return I(((a,b,d)=>{const e=Reflect.set(c(a),c(b),c(d));return e}),arguments)};b.wbg.__wbindgen_debug_string=((b,d)=>{const e=u(c(d));const f=o(e,a.__wbindgen_malloc,a.__wbindgen_realloc);const g=l;r()[b/a4+ U]=g;r()[b/a4+ Y]=f});b.wbg.__wbindgen_throw=((a,b)=>{throw new X(k(a,b))});b.wbg.__wbindgen_memory=(()=>{const b=a.memory;return g(b)});b.wbg.__wbindgen_closure_wrapper542=((a,b,c)=>{const d=w(a,b,a6,x);return g(d)});b.wbg.__wbindgen_closure_wrapper544=((a,b,c)=>{const d=w(a,b,a6,x);return g(d)});b.wbg.__wbindgen_closure_wrapper546=((a,b,c)=>{const d=w(a,b,a6,x);return g(d)});b.wbg.__wbindgen_closure_wrapper548=((a,b,c)=>{const d=w(a,b,a6,y);return g(d)});b.wbg.__wbindgen_closure_wrapper802=((a,b,c)=>{const d=z(a,b,409,C);return g(d)});b.wbg.__wbindgen_closure_wrapper1146=((a,b,c)=>{const d=w(a,b,507,D);return g(d)});b.wbg.__wbindgen_closure_wrapper1184=((a,b,c)=>{const d=w(a,b,529,E);return g(d)});return b});var t=(()=>{if(s===T||s.byteLength===Y){s=new Float64Array(a.memory.buffer)};return s});var J=((a,b)=>{a=a>>>Y;return j().subarray(a/U,a/U+ b)});var G=(()=>{if(F===T||F.byteLength===Y){F=new Uint32Array(a.memory.buffer)};return F});var o=((a,b,c)=>{if(c===S){const c=m.encode(a);const d=b(c.length,U)>>>Y;j().subarray(d,d+ c.length).set(c);l=c.length;return d};let d=a.length;let e=b(d,U)>>>Y;const f=j();let g=Y;for(;g<d;g++){const b=a.charCodeAt(g);if(b>127)break;f[e+ g]=b};if(g!==d){if(g!==Y){a=a.slice(g)};e=c(e,d,d=g+ a.length*3,U)>>>Y;const b=j().subarray(e+ g,e+ d);const f=n(a,b);g+=f.written;e=c(e,d,g,U)>>>Y};l=g;return e});var j=(()=>{if(i===T||i.byteLength===Y){i=new Z(a.memory.buffer)};return i});var K=(async(a,b)=>{if(typeof Response===_&&a instanceof Response){if(typeof WebAssembly.instantiateStreaming===_){try{return await WebAssembly.instantiateStreaming(a,b)}catch(b){if(a.headers.get(`Content-Type`)!=`application/wasm`){console.warn(`\`WebAssembly.instantiateStreaming\` failed because your server does not serve wasm with \`application/wasm\` MIME type. Falling back to \`WebAssembly.instantiate\` which is slower. Original error:\\n`,b)}else{throw b}}};const c=await a.arrayBuffer();return await WebAssembly.instantiate(c,b)}else{const c=await WebAssembly.instantiate(a,b);if(c instanceof WebAssembly.Instance){return {instance:c,module:a}}else{return c}}});var x=((b,c,d)=>{a._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h349c58db62d95828(b,c,g(d))});var H=((a,b)=>{a=a>>>Y;const c=G();const d=c.subarray(a/a4,a/a4+ b);const e=[];for(let a=Y;a<d.length;a++){e.push(f(d[a]))};return e});var g=(a=>{if(d===b.length)b.push(b.length+ U);const c=d;d=b[c];b[c]=a;return c});var f=(a=>{const b=c(a);e(a);return b});var r=(()=>{if(q===T||q.byteLength===Y){q=new Int32Array(a.memory.buffer)};return q});var E=((c,d,e)=>{try{a._dyn_core__ops__function__FnMut___A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h8f95d1b792021dd1(c,d,B(e))}finally{b[A++]=S}});var P=(async(b)=>{if(a!==S)return a;if(typeof b===V){b=new URL(`megabit-console-frontend-ea5288f650edb9cd_bg.wasm`,import.meta.url)};const c=L();if(typeof b===a1||typeof Request===_&&b instanceof Request||typeof URL===_&&b instanceof URL){b=fetch(b)};M(c);const {instance:d,module:e}=await K(await b,c);return N(d,e)});var O=(b=>{if(a!==S)return a;const c=L();M(c);if(!(b instanceof WebAssembly.Module)){b=new WebAssembly.Module(b)};const d=new WebAssembly.Instance(b,c);return N(d,b)});var z=((b,c,d,e)=>{const f={a:b,b:c,cnt:U,dtor:d};const g=(...b)=>{f.cnt++;try{return e(f.a,f.b,...b)}finally{if(--f.cnt===Y){a.__wbindgen_export_2.get(f.dtor)(f.a,f.b);f.a=Y;v.unregister(f)}}};g.original=f;v.register(g,f,f);return g});var e=(a=>{if(a<132)return;b[a]=d;d=a});var N=((b,c)=>{a=b.exports;P.__wbindgen_wasm_module=c;s=T;q=T;F=T;i=T;a.__wbindgen_start();return a});var C=((c,d,e)=>{try{a._dyn_core__ops__function__Fn___A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hed5e55af0fb0efaf(c,d,B(e))}finally{b[A++]=S}});var w=((b,c,d,e)=>{const f={a:b,b:c,cnt:U,dtor:d};const g=(...b)=>{f.cnt++;const c=f.a;f.a=Y;try{return e(c,f.b,...b)}finally{if(--f.cnt===Y){a.__wbindgen_export_2.get(f.dtor)(c,f.b);v.unregister(f)}else{f.a=c}}};g.original=f;v.register(g,f,f);return g});var k=((a,b)=>{a=a>>>Y;return h.decode(j().subarray(a,a+ b))});var B=(a=>{if(A==U)throw new X(`out of js stack`);b[--A]=a;return A});let a;const b=new Q(R).fill(S);b.push(S,T,!0,!1);let d=b.length;const h=typeof TextDecoder!==V?new TextDecoder(W,{ignoreBOM:!0,fatal:!0}):{decode:()=>{throw X(`TextDecoder not available`)}};if(typeof TextDecoder!==V){h.decode()};let i=T;let l=Y;const m=typeof TextEncoder!==V?new TextEncoder(W):{encode:()=>{throw X(`TextEncoder not available`)}};const n=typeof m.encodeInto===_?((a,b)=>m.encodeInto(a,b)):((a,b)=>{const c=m.encode(a);b.set(c);return {read:a.length,written:c.length}});let q=T;let s=T;const v=typeof a3===V?{register:()=>{},unregister:()=>{}}:new a3(b=>{a.__wbindgen_export_2.get(b.dtor)(b.a,b.b)});let A=R;let F=T;export default P;export{O as initSync}