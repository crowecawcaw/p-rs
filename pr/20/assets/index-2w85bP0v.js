var LS=Object.defineProperty;var DS=(t,e,n)=>e in t?LS(t,e,{enumerable:!0,configurable:!0,writable:!0,value:n}):t[e]=n;var Oo=(t,e,n)=>DS(t,typeof e!="symbol"?e+"":e,n);(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const i of document.querySelectorAll('link[rel="modulepreload"]'))r(i);new MutationObserver(i=>{for(const s of i)if(s.type==="childList")for(const o of s.addedNodes)o.tagName==="LINK"&&o.rel==="modulepreload"&&r(o)}).observe(document,{childList:!0,subtree:!0});function n(i){const s={};return i.integrity&&(s.integrity=i.integrity),i.referrerPolicy&&(s.referrerPolicy=i.referrerPolicy),i.crossOrigin==="use-credentials"?s.credentials="include":i.crossOrigin==="anonymous"?s.credentials="omit":s.credentials="same-origin",s}function r(i){if(i.ep)return;i.ep=!0;const s=n(i);fetch(i.href,s)}})();var Qg={exports:{}},Na={},Yg={exports:{}},J={};/**
 * @license React
 * react.production.min.js
 *
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */var xo=Symbol.for("react.element"),OS=Symbol.for("react.portal"),NS=Symbol.for("react.fragment"),BS=Symbol.for("react.strict_mode"),_S=Symbol.for("react.profiler"),qS=Symbol.for("react.provider"),WS=Symbol.for("react.context"),FS=Symbol.for("react.forward_ref"),zS=Symbol.for("react.suspense"),jS=Symbol.for("react.memo"),VS=Symbol.for("react.lazy"),Xd=Symbol.iterator;function HS(t){return t===null||typeof t!="object"?null:(t=Xd&&t[Xd]||t["@@iterator"],typeof t=="function"?t:null)}var Xg={isMounted:function(){return!1},enqueueForceUpdate:function(){},enqueueReplaceState:function(){},enqueueSetState:function(){}},Jg=Object.assign,Zg={};function Xi(t,e,n){this.props=t,this.context=e,this.refs=Zg,this.updater=n||Xg}Xi.prototype.isReactComponent={};Xi.prototype.setState=function(t,e){if(typeof t!="object"&&typeof t!="function"&&t!=null)throw Error("setState(...): takes an object of state variables to update or a function which returns an object of state variables.");this.updater.enqueueSetState(this,t,e,"setState")};Xi.prototype.forceUpdate=function(t){this.updater.enqueueForceUpdate(this,t,"forceUpdate")};function e0(){}e0.prototype=Xi.prototype;function hf(t,e,n){this.props=t,this.context=e,this.refs=Zg,this.updater=n||Xg}var ff=hf.prototype=new e0;ff.constructor=hf;Jg(ff,Xi.prototype);ff.isPureReactComponent=!0;var Jd=Array.isArray,t0=Object.prototype.hasOwnProperty,df={current:null},n0={key:!0,ref:!0,__self:!0,__source:!0};function r0(t,e,n){var r,i={},s=null,o=null;if(e!=null)for(r in e.ref!==void 0&&(o=e.ref),e.key!==void 0&&(s=""+e.key),e)t0.call(e,r)&&!n0.hasOwnProperty(r)&&(i[r]=e[r]);var l=arguments.length-2;if(l===1)i.children=n;else if(1<l){for(var a=Array(l),c=0;c<l;c++)a[c]=arguments[c+2];i.children=a}if(t&&t.defaultProps)for(r in l=t.defaultProps,l)i[r]===void 0&&(i[r]=l[r]);return{$$typeof:xo,type:t,key:s,ref:o,props:i,_owner:df.current}}function US(t,e){return{$$typeof:xo,type:t.type,key:e,ref:t.ref,props:t.props,_owner:t._owner}}function pf(t){return typeof t=="object"&&t!==null&&t.$$typeof===xo}function $S(t){var e={"=":"=0",":":"=2"};return"$"+t.replace(/[=:]/g,function(n){return e[n]})}var Zd=/\/+/g;function yc(t,e){return typeof t=="object"&&t!==null&&t.key!=null?$S(""+t.key):e.toString(36)}function kl(t,e,n,r,i){var s=typeof t;(s==="undefined"||s==="boolean")&&(t=null);var o=!1;if(t===null)o=!0;else switch(s){case"string":case"number":o=!0;break;case"object":switch(t.$$typeof){case xo:case OS:o=!0}}if(o)return o=t,i=i(o),t=r===""?"."+yc(o,0):r,Jd(i)?(n="",t!=null&&(n=t.replace(Zd,"$&/")+"/"),kl(i,e,n,"",function(c){return c})):i!=null&&(pf(i)&&(i=US(i,n+(!i.key||o&&o.key===i.key?"":(""+i.key).replace(Zd,"$&/")+"/")+t)),e.push(i)),1;if(o=0,r=r===""?".":r+":",Jd(t))for(var l=0;l<t.length;l++){s=t[l];var a=r+yc(s,l);o+=kl(s,e,n,a,i)}else if(a=HS(t),typeof a=="function")for(t=a.call(t),l=0;!(s=t.next()).done;)s=s.value,a=r+yc(s,l++),o+=kl(s,e,n,a,i);else if(s==="object")throw e=String(t),Error("Objects are not valid as a React child (found: "+(e==="[object Object]"?"object with keys {"+Object.keys(t).join(", ")+"}":e)+"). If you meant to render a collection of children, use an array instead.");return o}function No(t,e,n){if(t==null)return t;var r=[],i=0;return kl(t,r,"","",function(s){return e.call(n,s,i++)}),r}function GS(t){if(t._status===-1){var e=t._result;e=e(),e.then(function(n){(t._status===0||t._status===-1)&&(t._status=1,t._result=n)},function(n){(t._status===0||t._status===-1)&&(t._status=2,t._result=n)}),t._status===-1&&(t._status=0,t._result=e)}if(t._status===1)return t._result.default;throw t._result}var ct={current:null},xl={transition:null},KS={ReactCurrentDispatcher:ct,ReactCurrentBatchConfig:xl,ReactCurrentOwner:df};function i0(){throw Error("act(...) is not supported in production builds of React.")}J.Children={map:No,forEach:function(t,e,n){No(t,function(){e.apply(this,arguments)},n)},count:function(t){var e=0;return No(t,function(){e++}),e},toArray:function(t){return No(t,function(e){return e})||[]},only:function(t){if(!pf(t))throw Error("React.Children.only expected to receive a single React element child.");return t}};J.Component=Xi;J.Fragment=NS;J.Profiler=_S;J.PureComponent=hf;J.StrictMode=BS;J.Suspense=zS;J.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED=KS;J.act=i0;J.cloneElement=function(t,e,n){if(t==null)throw Error("React.cloneElement(...): The argument must be a React element, but you passed "+t+".");var r=Jg({},t.props),i=t.key,s=t.ref,o=t._owner;if(e!=null){if(e.ref!==void 0&&(s=e.ref,o=df.current),e.key!==void 0&&(i=""+e.key),t.type&&t.type.defaultProps)var l=t.type.defaultProps;for(a in e)t0.call(e,a)&&!n0.hasOwnProperty(a)&&(r[a]=e[a]===void 0&&l!==void 0?l[a]:e[a])}var a=arguments.length-2;if(a===1)r.children=n;else if(1<a){l=Array(a);for(var c=0;c<a;c++)l[c]=arguments[c+2];r.children=l}return{$$typeof:xo,type:t.type,key:i,ref:s,props:r,_owner:o}};J.createContext=function(t){return t={$$typeof:WS,_currentValue:t,_currentValue2:t,_threadCount:0,Provider:null,Consumer:null,_defaultValue:null,_globalName:null},t.Provider={$$typeof:qS,_context:t},t.Consumer=t};J.createElement=r0;J.createFactory=function(t){var e=r0.bind(null,t);return e.type=t,e};J.createRef=function(){return{current:null}};J.forwardRef=function(t){return{$$typeof:FS,render:t}};J.isValidElement=pf;J.lazy=function(t){return{$$typeof:VS,_payload:{_status:-1,_result:t},_init:GS}};J.memo=function(t,e){return{$$typeof:jS,type:t,compare:e===void 0?null:e}};J.startTransition=function(t){var e=xl.transition;xl.transition={};try{t()}finally{xl.transition=e}};J.unstable_act=i0;J.useCallback=function(t,e){return ct.current.useCallback(t,e)};J.useContext=function(t){return ct.current.useContext(t)};J.useDebugValue=function(){};J.useDeferredValue=function(t){return ct.current.useDeferredValue(t)};J.useEffect=function(t,e){return ct.current.useEffect(t,e)};J.useId=function(){return ct.current.useId()};J.useImperativeHandle=function(t,e,n){return ct.current.useImperativeHandle(t,e,n)};J.useInsertionEffect=function(t,e){return ct.current.useInsertionEffect(t,e)};J.useLayoutEffect=function(t,e){return ct.current.useLayoutEffect(t,e)};J.useMemo=function(t,e){return ct.current.useMemo(t,e)};J.useReducer=function(t,e,n){return ct.current.useReducer(t,e,n)};J.useRef=function(t){return ct.current.useRef(t)};J.useState=function(t){return ct.current.useState(t)};J.useSyncExternalStore=function(t,e,n){return ct.current.useSyncExternalStore(t,e,n)};J.useTransition=function(){return ct.current.useTransition()};J.version="18.3.1";Yg.exports=J;var O=Yg.exports;/**
 * @license React
 * react-jsx-runtime.production.min.js
 *
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */var QS=O,YS=Symbol.for("react.element"),XS=Symbol.for("react.fragment"),JS=Object.prototype.hasOwnProperty,ZS=QS.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED.ReactCurrentOwner,ek={key:!0,ref:!0,__self:!0,__source:!0};function s0(t,e,n){var r,i={},s=null,o=null;n!==void 0&&(s=""+n),e.key!==void 0&&(s=""+e.key),e.ref!==void 0&&(o=e.ref);for(r in e)JS.call(e,r)&&!ek.hasOwnProperty(r)&&(i[r]=e[r]);if(t&&t.defaultProps)for(r in e=t.defaultProps,e)i[r]===void 0&&(i[r]=e[r]);return{$$typeof:YS,type:t,key:s,ref:o,props:i,_owner:ZS.current}}Na.Fragment=XS;Na.jsx=s0;Na.jsxs=s0;Qg.exports=Na;var S=Qg.exports,o0={exports:{}},It={},l0={exports:{}},a0={};/**
 * @license React
 * scheduler.production.min.js
 *
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */(function(t){function e(T,A){var P=T.length;T.push(A);e:for(;0<P;){var H=P-1>>>1,G=T[H];if(0<i(G,A))T[H]=A,T[P]=G,P=H;else break e}}function n(T){return T.length===0?null:T[0]}function r(T){if(T.length===0)return null;var A=T[0],P=T.pop();if(P!==A){T[0]=P;e:for(var H=0,G=T.length,re=G>>>1;H<re;){var pe=2*(H+1)-1,ke=T[pe],se=pe+1,je=T[se];if(0>i(ke,P))se<G&&0>i(je,ke)?(T[H]=je,T[se]=P,H=se):(T[H]=ke,T[pe]=P,H=pe);else if(se<G&&0>i(je,P))T[H]=je,T[se]=P,H=se;else break e}}return A}function i(T,A){var P=T.sortIndex-A.sortIndex;return P!==0?P:T.id-A.id}if(typeof performance=="object"&&typeof performance.now=="function"){var s=performance;t.unstable_now=function(){return s.now()}}else{var o=Date,l=o.now();t.unstable_now=function(){return o.now()-l}}var a=[],c=[],u=1,h=null,f=3,d=!1,p=!1,m=!1,w=typeof setTimeout=="function"?setTimeout:null,g=typeof clearTimeout=="function"?clearTimeout:null,v=typeof setImmediate<"u"?setImmediate:null;typeof navigator<"u"&&navigator.scheduling!==void 0&&navigator.scheduling.isInputPending!==void 0&&navigator.scheduling.isInputPending.bind(navigator.scheduling);function y(T){for(var A=n(c);A!==null;){if(A.callback===null)r(c);else if(A.startTime<=T)r(c),A.sortIndex=A.expirationTime,e(a,A);else break;A=n(c)}}function x(T){if(m=!1,y(T),!p)if(n(a)!==null)p=!0,de(k);else{var A=n(c);A!==null&&ae(x,A.startTime-T)}}function k(T,A){p=!1,m&&(m=!1,g(I),I=-1),d=!0;var P=f;try{for(y(A),h=n(a);h!==null&&(!(h.expirationTime>A)||T&&!q());){var H=h.callback;if(typeof H=="function"){h.callback=null,f=h.priorityLevel;var G=H(h.expirationTime<=A);A=t.unstable_now(),typeof G=="function"?h.callback=G:h===n(a)&&r(a),y(A)}else r(a);h=n(a)}if(h!==null)var re=!0;else{var pe=n(c);pe!==null&&ae(x,pe.startTime-A),re=!1}return re}finally{h=null,f=P,d=!1}}var C=!1,b=null,I=-1,W=5,N=-1;function q(){return!(t.unstable_now()-N<W)}function j(){if(b!==null){var T=t.unstable_now();N=T;var A=!0;try{A=b(!0,T)}finally{A?M():(C=!1,b=null)}}else C=!1}var M;if(typeof v=="function")M=function(){v(j)};else if(typeof MessageChannel<"u"){var $=new MessageChannel,Q=$.port2;$.port1.onmessage=j,M=function(){Q.postMessage(null)}}else M=function(){w(j,0)};function de(T){b=T,C||(C=!0,M())}function ae(T,A){I=w(function(){T(t.unstable_now())},A)}t.unstable_IdlePriority=5,t.unstable_ImmediatePriority=1,t.unstable_LowPriority=4,t.unstable_NormalPriority=3,t.unstable_Profiling=null,t.unstable_UserBlockingPriority=2,t.unstable_cancelCallback=function(T){T.callback=null},t.unstable_continueExecution=function(){p||d||(p=!0,de(k))},t.unstable_forceFrameRate=function(T){0>T||125<T?console.error("forceFrameRate takes a positive int between 0 and 125, forcing frame rates higher than 125 fps is not supported"):W=0<T?Math.floor(1e3/T):5},t.unstable_getCurrentPriorityLevel=function(){return f},t.unstable_getFirstCallbackNode=function(){return n(a)},t.unstable_next=function(T){switch(f){case 1:case 2:case 3:var A=3;break;default:A=f}var P=f;f=A;try{return T()}finally{f=P}},t.unstable_pauseExecution=function(){},t.unstable_requestPaint=function(){},t.unstable_runWithPriority=function(T,A){switch(T){case 1:case 2:case 3:case 4:case 5:break;default:T=3}var P=f;f=T;try{return A()}finally{f=P}},t.unstable_scheduleCallback=function(T,A,P){var H=t.unstable_now();switch(typeof P=="object"&&P!==null?(P=P.delay,P=typeof P=="number"&&0<P?H+P:H):P=H,T){case 1:var G=-1;break;case 2:G=250;break;case 5:G=1073741823;break;case 4:G=1e4;break;default:G=5e3}return G=P+G,T={id:u++,callback:A,priorityLevel:T,startTime:P,expirationTime:G,sortIndex:-1},P>H?(T.sortIndex=P,e(c,T),n(a)===null&&T===n(c)&&(m?(g(I),I=-1):m=!0,ae(x,P-H))):(T.sortIndex=G,e(a,T),p||d||(p=!0,de(k))),T},t.unstable_shouldYield=q,t.unstable_wrapCallback=function(T){var A=f;return function(){var P=f;f=A;try{return T.apply(this,arguments)}finally{f=P}}}})(a0);l0.exports=a0;var tk=l0.exports;/**
 * @license React
 * react-dom.production.min.js
 *
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */var nk=O,Mt=tk;function E(t){for(var e="https://reactjs.org/docs/error-decoder.html?invariant="+t,n=1;n<arguments.length;n++)e+="&args[]="+encodeURIComponent(arguments[n]);return"Minified React error #"+t+"; visit "+e+" for the full message or use the non-minified dev environment for full errors and additional helpful warnings."}var c0=new Set,Vs={};function Xr(t,e){Ni(t,e),Ni(t+"Capture",e)}function Ni(t,e){for(Vs[t]=e,t=0;t<e.length;t++)c0.add(e[t])}var En=!(typeof window>"u"||typeof window.document>"u"||typeof window.document.createElement>"u"),xu=Object.prototype.hasOwnProperty,rk=/^[:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD][:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\-.0-9\u00B7\u0300-\u036F\u203F-\u2040]*$/,ep={},tp={};function ik(t){return xu.call(tp,t)?!0:xu.call(ep,t)?!1:rk.test(t)?tp[t]=!0:(ep[t]=!0,!1)}function sk(t,e,n,r){if(n!==null&&n.type===0)return!1;switch(typeof e){case"function":case"symbol":return!0;case"boolean":return r?!1:n!==null?!n.acceptsBooleans:(t=t.toLowerCase().slice(0,5),t!=="data-"&&t!=="aria-");default:return!1}}function ok(t,e,n,r){if(e===null||typeof e>"u"||sk(t,e,n,r))return!0;if(r)return!1;if(n!==null)switch(n.type){case 3:return!e;case 4:return e===!1;case 5:return isNaN(e);case 6:return isNaN(e)||1>e}return!1}function ut(t,e,n,r,i,s,o){this.acceptsBooleans=e===2||e===3||e===4,this.attributeName=r,this.attributeNamespace=i,this.mustUseProperty=n,this.propertyName=t,this.type=e,this.sanitizeURL=s,this.removeEmptyString=o}var Ke={};"children dangerouslySetInnerHTML defaultValue defaultChecked innerHTML suppressContentEditableWarning suppressHydrationWarning style".split(" ").forEach(function(t){Ke[t]=new ut(t,0,!1,t,null,!1,!1)});[["acceptCharset","accept-charset"],["className","class"],["htmlFor","for"],["httpEquiv","http-equiv"]].forEach(function(t){var e=t[0];Ke[e]=new ut(e,1,!1,t[1],null,!1,!1)});["contentEditable","draggable","spellCheck","value"].forEach(function(t){Ke[t]=new ut(t,2,!1,t.toLowerCase(),null,!1,!1)});["autoReverse","externalResourcesRequired","focusable","preserveAlpha"].forEach(function(t){Ke[t]=new ut(t,2,!1,t,null,!1,!1)});"allowFullScreen async autoFocus autoPlay controls default defer disabled disablePictureInPicture disableRemotePlayback formNoValidate hidden loop noModule noValidate open playsInline readOnly required reversed scoped seamless itemScope".split(" ").forEach(function(t){Ke[t]=new ut(t,3,!1,t.toLowerCase(),null,!1,!1)});["checked","multiple","muted","selected"].forEach(function(t){Ke[t]=new ut(t,3,!0,t,null,!1,!1)});["capture","download"].forEach(function(t){Ke[t]=new ut(t,4,!1,t,null,!1,!1)});["cols","rows","size","span"].forEach(function(t){Ke[t]=new ut(t,6,!1,t,null,!1,!1)});["rowSpan","start"].forEach(function(t){Ke[t]=new ut(t,5,!1,t.toLowerCase(),null,!1,!1)});var mf=/[\-:]([a-z])/g;function gf(t){return t[1].toUpperCase()}"accent-height alignment-baseline arabic-form baseline-shift cap-height clip-path clip-rule color-interpolation color-interpolation-filters color-profile color-rendering dominant-baseline enable-background fill-opacity fill-rule flood-color flood-opacity font-family font-size font-size-adjust font-stretch font-style font-variant font-weight glyph-name glyph-orientation-horizontal glyph-orientation-vertical horiz-adv-x horiz-origin-x image-rendering letter-spacing lighting-color marker-end marker-mid marker-start overline-position overline-thickness paint-order panose-1 pointer-events rendering-intent shape-rendering stop-color stop-opacity strikethrough-position strikethrough-thickness stroke-dasharray stroke-dashoffset stroke-linecap stroke-linejoin stroke-miterlimit stroke-opacity stroke-width text-anchor text-decoration text-rendering underline-position underline-thickness unicode-bidi unicode-range units-per-em v-alphabetic v-hanging v-ideographic v-mathematical vector-effect vert-adv-y vert-origin-x vert-origin-y word-spacing writing-mode xmlns:xlink x-height".split(" ").forEach(function(t){var e=t.replace(mf,gf);Ke[e]=new ut(e,1,!1,t,null,!1,!1)});"xlink:actuate xlink:arcrole xlink:role xlink:show xlink:title xlink:type".split(" ").forEach(function(t){var e=t.replace(mf,gf);Ke[e]=new ut(e,1,!1,t,"http://www.w3.org/1999/xlink",!1,!1)});["xml:base","xml:lang","xml:space"].forEach(function(t){var e=t.replace(mf,gf);Ke[e]=new ut(e,1,!1,t,"http://www.w3.org/XML/1998/namespace",!1,!1)});["tabIndex","crossOrigin"].forEach(function(t){Ke[t]=new ut(t,1,!1,t.toLowerCase(),null,!1,!1)});Ke.xlinkHref=new ut("xlinkHref",1,!1,"xlink:href","http://www.w3.org/1999/xlink",!0,!1);["src","href","action","formAction"].forEach(function(t){Ke[t]=new ut(t,1,!1,t.toLowerCase(),null,!0,!0)});function vf(t,e,n,r){var i=Ke.hasOwnProperty(e)?Ke[e]:null;(i!==null?i.type!==0:r||!(2<e.length)||e[0]!=="o"&&e[0]!=="O"||e[1]!=="n"&&e[1]!=="N")&&(ok(e,n,i,r)&&(n=null),r||i===null?ik(e)&&(n===null?t.removeAttribute(e):t.setAttribute(e,""+n)):i.mustUseProperty?t[i.propertyName]=n===null?i.type===3?!1:"":n:(e=i.attributeName,r=i.attributeNamespace,n===null?t.removeAttribute(e):(i=i.type,n=i===3||i===4&&n===!0?"":""+n,r?t.setAttributeNS(r,e,n):t.setAttribute(e,n))))}var Bn=nk.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED,Bo=Symbol.for("react.element"),oi=Symbol.for("react.portal"),li=Symbol.for("react.fragment"),yf=Symbol.for("react.strict_mode"),Cu=Symbol.for("react.profiler"),u0=Symbol.for("react.provider"),h0=Symbol.for("react.context"),wf=Symbol.for("react.forward_ref"),bu=Symbol.for("react.suspense"),Tu=Symbol.for("react.suspense_list"),Sf=Symbol.for("react.memo"),Fn=Symbol.for("react.lazy"),f0=Symbol.for("react.offscreen"),np=Symbol.iterator;function ss(t){return t===null||typeof t!="object"?null:(t=np&&t[np]||t["@@iterator"],typeof t=="function"?t:null)}var be=Object.assign,wc;function ws(t){if(wc===void 0)try{throw Error()}catch(n){var e=n.stack.trim().match(/\n( *(at )?)/);wc=e&&e[1]||""}return`
`+wc+t}var Sc=!1;function kc(t,e){if(!t||Sc)return"";Sc=!0;var n=Error.prepareStackTrace;Error.prepareStackTrace=void 0;try{if(e)if(e=function(){throw Error()},Object.defineProperty(e.prototype,"props",{set:function(){throw Error()}}),typeof Reflect=="object"&&Reflect.construct){try{Reflect.construct(e,[])}catch(c){var r=c}Reflect.construct(t,[],e)}else{try{e.call()}catch(c){r=c}t.call(e.prototype)}else{try{throw Error()}catch(c){r=c}t()}}catch(c){if(c&&r&&typeof c.stack=="string"){for(var i=c.stack.split(`
`),s=r.stack.split(`
`),o=i.length-1,l=s.length-1;1<=o&&0<=l&&i[o]!==s[l];)l--;for(;1<=o&&0<=l;o--,l--)if(i[o]!==s[l]){if(o!==1||l!==1)do if(o--,l--,0>l||i[o]!==s[l]){var a=`
`+i[o].replace(" at new "," at ");return t.displayName&&a.includes("<anonymous>")&&(a=a.replace("<anonymous>",t.displayName)),a}while(1<=o&&0<=l);break}}}finally{Sc=!1,Error.prepareStackTrace=n}return(t=t?t.displayName||t.name:"")?ws(t):""}function lk(t){switch(t.tag){case 5:return ws(t.type);case 16:return ws("Lazy");case 13:return ws("Suspense");case 19:return ws("SuspenseList");case 0:case 2:case 15:return t=kc(t.type,!1),t;case 11:return t=kc(t.type.render,!1),t;case 1:return t=kc(t.type,!0),t;default:return""}}function Ru(t){if(t==null)return null;if(typeof t=="function")return t.displayName||t.name||null;if(typeof t=="string")return t;switch(t){case li:return"Fragment";case oi:return"Portal";case Cu:return"Profiler";case yf:return"StrictMode";case bu:return"Suspense";case Tu:return"SuspenseList"}if(typeof t=="object")switch(t.$$typeof){case h0:return(t.displayName||"Context")+".Consumer";case u0:return(t._context.displayName||"Context")+".Provider";case wf:var e=t.render;return t=t.displayName,t||(t=e.displayName||e.name||"",t=t!==""?"ForwardRef("+t+")":"ForwardRef"),t;case Sf:return e=t.displayName||null,e!==null?e:Ru(t.type)||"Memo";case Fn:e=t._payload,t=t._init;try{return Ru(t(e))}catch{}}return null}function ak(t){var e=t.type;switch(t.tag){case 24:return"Cache";case 9:return(e.displayName||"Context")+".Consumer";case 10:return(e._context.displayName||"Context")+".Provider";case 18:return"DehydratedFragment";case 11:return t=e.render,t=t.displayName||t.name||"",e.displayName||(t!==""?"ForwardRef("+t+")":"ForwardRef");case 7:return"Fragment";case 5:return e;case 4:return"Portal";case 3:return"Root";case 6:return"Text";case 16:return Ru(e);case 8:return e===yf?"StrictMode":"Mode";case 22:return"Offscreen";case 12:return"Profiler";case 21:return"Scope";case 13:return"Suspense";case 19:return"SuspenseList";case 25:return"TracingMarker";case 1:case 0:case 17:case 2:case 14:case 15:if(typeof e=="function")return e.displayName||e.name||null;if(typeof e=="string")return e}return null}function ar(t){switch(typeof t){case"boolean":case"number":case"string":case"undefined":return t;case"object":return t;default:return""}}function d0(t){var e=t.type;return(t=t.nodeName)&&t.toLowerCase()==="input"&&(e==="checkbox"||e==="radio")}function ck(t){var e=d0(t)?"checked":"value",n=Object.getOwnPropertyDescriptor(t.constructor.prototype,e),r=""+t[e];if(!t.hasOwnProperty(e)&&typeof n<"u"&&typeof n.get=="function"&&typeof n.set=="function"){var i=n.get,s=n.set;return Object.defineProperty(t,e,{configurable:!0,get:function(){return i.call(this)},set:function(o){r=""+o,s.call(this,o)}}),Object.defineProperty(t,e,{enumerable:n.enumerable}),{getValue:function(){return r},setValue:function(o){r=""+o},stopTracking:function(){t._valueTracker=null,delete t[e]}}}}function _o(t){t._valueTracker||(t._valueTracker=ck(t))}function p0(t){if(!t)return!1;var e=t._valueTracker;if(!e)return!0;var n=e.getValue(),r="";return t&&(r=d0(t)?t.checked?"true":"false":t.value),t=r,t!==n?(e.setValue(t),!0):!1}function Vl(t){if(t=t||(typeof document<"u"?document:void 0),typeof t>"u")return null;try{return t.activeElement||t.body}catch{return t.body}}function Mu(t,e){var n=e.checked;return be({},e,{defaultChecked:void 0,defaultValue:void 0,value:void 0,checked:n??t._wrapperState.initialChecked})}function rp(t,e){var n=e.defaultValue==null?"":e.defaultValue,r=e.checked!=null?e.checked:e.defaultChecked;n=ar(e.value!=null?e.value:n),t._wrapperState={initialChecked:r,initialValue:n,controlled:e.type==="checkbox"||e.type==="radio"?e.checked!=null:e.value!=null}}function m0(t,e){e=e.checked,e!=null&&vf(t,"checked",e,!1)}function Iu(t,e){m0(t,e);var n=ar(e.value),r=e.type;if(n!=null)r==="number"?(n===0&&t.value===""||t.value!=n)&&(t.value=""+n):t.value!==""+n&&(t.value=""+n);else if(r==="submit"||r==="reset"){t.removeAttribute("value");return}e.hasOwnProperty("value")?Au(t,e.type,n):e.hasOwnProperty("defaultValue")&&Au(t,e.type,ar(e.defaultValue)),e.checked==null&&e.defaultChecked!=null&&(t.defaultChecked=!!e.defaultChecked)}function ip(t,e,n){if(e.hasOwnProperty("value")||e.hasOwnProperty("defaultValue")){var r=e.type;if(!(r!=="submit"&&r!=="reset"||e.value!==void 0&&e.value!==null))return;e=""+t._wrapperState.initialValue,n||e===t.value||(t.value=e),t.defaultValue=e}n=t.name,n!==""&&(t.name=""),t.defaultChecked=!!t._wrapperState.initialChecked,n!==""&&(t.name=n)}function Au(t,e,n){(e!=="number"||Vl(t.ownerDocument)!==t)&&(n==null?t.defaultValue=""+t._wrapperState.initialValue:t.defaultValue!==""+n&&(t.defaultValue=""+n))}var Ss=Array.isArray;function xi(t,e,n,r){if(t=t.options,e){e={};for(var i=0;i<n.length;i++)e["$"+n[i]]=!0;for(n=0;n<t.length;n++)i=e.hasOwnProperty("$"+t[n].value),t[n].selected!==i&&(t[n].selected=i),i&&r&&(t[n].defaultSelected=!0)}else{for(n=""+ar(n),e=null,i=0;i<t.length;i++){if(t[i].value===n){t[i].selected=!0,r&&(t[i].defaultSelected=!0);return}e!==null||t[i].disabled||(e=t[i])}e!==null&&(e.selected=!0)}}function Pu(t,e){if(e.dangerouslySetInnerHTML!=null)throw Error(E(91));return be({},e,{value:void 0,defaultValue:void 0,children:""+t._wrapperState.initialValue})}function sp(t,e){var n=e.value;if(n==null){if(n=e.children,e=e.defaultValue,n!=null){if(e!=null)throw Error(E(92));if(Ss(n)){if(1<n.length)throw Error(E(93));n=n[0]}e=n}e==null&&(e=""),n=e}t._wrapperState={initialValue:ar(n)}}function g0(t,e){var n=ar(e.value),r=ar(e.defaultValue);n!=null&&(n=""+n,n!==t.value&&(t.value=n),e.defaultValue==null&&t.defaultValue!==n&&(t.defaultValue=n)),r!=null&&(t.defaultValue=""+r)}function op(t){var e=t.textContent;e===t._wrapperState.initialValue&&e!==""&&e!==null&&(t.value=e)}function v0(t){switch(t){case"svg":return"http://www.w3.org/2000/svg";case"math":return"http://www.w3.org/1998/Math/MathML";default:return"http://www.w3.org/1999/xhtml"}}function Eu(t,e){return t==null||t==="http://www.w3.org/1999/xhtml"?v0(e):t==="http://www.w3.org/2000/svg"&&e==="foreignObject"?"http://www.w3.org/1999/xhtml":t}var qo,y0=function(t){return typeof MSApp<"u"&&MSApp.execUnsafeLocalFunction?function(e,n,r,i){MSApp.execUnsafeLocalFunction(function(){return t(e,n,r,i)})}:t}(function(t,e){if(t.namespaceURI!=="http://www.w3.org/2000/svg"||"innerHTML"in t)t.innerHTML=e;else{for(qo=qo||document.createElement("div"),qo.innerHTML="<svg>"+e.valueOf().toString()+"</svg>",e=qo.firstChild;t.firstChild;)t.removeChild(t.firstChild);for(;e.firstChild;)t.appendChild(e.firstChild)}});function Hs(t,e){if(e){var n=t.firstChild;if(n&&n===t.lastChild&&n.nodeType===3){n.nodeValue=e;return}}t.textContent=e}var Rs={animationIterationCount:!0,aspectRatio:!0,borderImageOutset:!0,borderImageSlice:!0,borderImageWidth:!0,boxFlex:!0,boxFlexGroup:!0,boxOrdinalGroup:!0,columnCount:!0,columns:!0,flex:!0,flexGrow:!0,flexPositive:!0,flexShrink:!0,flexNegative:!0,flexOrder:!0,gridArea:!0,gridRow:!0,gridRowEnd:!0,gridRowSpan:!0,gridRowStart:!0,gridColumn:!0,gridColumnEnd:!0,gridColumnSpan:!0,gridColumnStart:!0,fontWeight:!0,lineClamp:!0,lineHeight:!0,opacity:!0,order:!0,orphans:!0,tabSize:!0,widows:!0,zIndex:!0,zoom:!0,fillOpacity:!0,floodOpacity:!0,stopOpacity:!0,strokeDasharray:!0,strokeDashoffset:!0,strokeMiterlimit:!0,strokeOpacity:!0,strokeWidth:!0},uk=["Webkit","ms","Moz","O"];Object.keys(Rs).forEach(function(t){uk.forEach(function(e){e=e+t.charAt(0).toUpperCase()+t.substring(1),Rs[e]=Rs[t]})});function w0(t,e,n){return e==null||typeof e=="boolean"||e===""?"":n||typeof e!="number"||e===0||Rs.hasOwnProperty(t)&&Rs[t]?(""+e).trim():e+"px"}function S0(t,e){t=t.style;for(var n in e)if(e.hasOwnProperty(n)){var r=n.indexOf("--")===0,i=w0(n,e[n],r);n==="float"&&(n="cssFloat"),r?t.setProperty(n,i):t[n]=i}}var hk=be({menuitem:!0},{area:!0,base:!0,br:!0,col:!0,embed:!0,hr:!0,img:!0,input:!0,keygen:!0,link:!0,meta:!0,param:!0,source:!0,track:!0,wbr:!0});function Lu(t,e){if(e){if(hk[t]&&(e.children!=null||e.dangerouslySetInnerHTML!=null))throw Error(E(137,t));if(e.dangerouslySetInnerHTML!=null){if(e.children!=null)throw Error(E(60));if(typeof e.dangerouslySetInnerHTML!="object"||!("__html"in e.dangerouslySetInnerHTML))throw Error(E(61))}if(e.style!=null&&typeof e.style!="object")throw Error(E(62))}}function Du(t,e){if(t.indexOf("-")===-1)return typeof e.is=="string";switch(t){case"annotation-xml":case"color-profile":case"font-face":case"font-face-src":case"font-face-uri":case"font-face-format":case"font-face-name":case"missing-glyph":return!1;default:return!0}}var Ou=null;function kf(t){return t=t.target||t.srcElement||window,t.correspondingUseElement&&(t=t.correspondingUseElement),t.nodeType===3?t.parentNode:t}var Nu=null,Ci=null,bi=null;function lp(t){if(t=To(t)){if(typeof Nu!="function")throw Error(E(280));var e=t.stateNode;e&&(e=Fa(e),Nu(t.stateNode,t.type,e))}}function k0(t){Ci?bi?bi.push(t):bi=[t]:Ci=t}function x0(){if(Ci){var t=Ci,e=bi;if(bi=Ci=null,lp(t),e)for(t=0;t<e.length;t++)lp(e[t])}}function C0(t,e){return t(e)}function b0(){}var xc=!1;function T0(t,e,n){if(xc)return t(e,n);xc=!0;try{return C0(t,e,n)}finally{xc=!1,(Ci!==null||bi!==null)&&(b0(),x0())}}function Us(t,e){var n=t.stateNode;if(n===null)return null;var r=Fa(n);if(r===null)return null;n=r[e];e:switch(e){case"onClick":case"onClickCapture":case"onDoubleClick":case"onDoubleClickCapture":case"onMouseDown":case"onMouseDownCapture":case"onMouseMove":case"onMouseMoveCapture":case"onMouseUp":case"onMouseUpCapture":case"onMouseEnter":(r=!r.disabled)||(t=t.type,r=!(t==="button"||t==="input"||t==="select"||t==="textarea")),t=!r;break e;default:t=!1}if(t)return null;if(n&&typeof n!="function")throw Error(E(231,e,typeof n));return n}var Bu=!1;if(En)try{var os={};Object.defineProperty(os,"passive",{get:function(){Bu=!0}}),window.addEventListener("test",os,os),window.removeEventListener("test",os,os)}catch{Bu=!1}function fk(t,e,n,r,i,s,o,l,a){var c=Array.prototype.slice.call(arguments,3);try{e.apply(n,c)}catch(u){this.onError(u)}}var Ms=!1,Hl=null,Ul=!1,_u=null,dk={onError:function(t){Ms=!0,Hl=t}};function pk(t,e,n,r,i,s,o,l,a){Ms=!1,Hl=null,fk.apply(dk,arguments)}function mk(t,e,n,r,i,s,o,l,a){if(pk.apply(this,arguments),Ms){if(Ms){var c=Hl;Ms=!1,Hl=null}else throw Error(E(198));Ul||(Ul=!0,_u=c)}}function Jr(t){var e=t,n=t;if(t.alternate)for(;e.return;)e=e.return;else{t=e;do e=t,e.flags&4098&&(n=e.return),t=e.return;while(t)}return e.tag===3?n:null}function R0(t){if(t.tag===13){var e=t.memoizedState;if(e===null&&(t=t.alternate,t!==null&&(e=t.memoizedState)),e!==null)return e.dehydrated}return null}function ap(t){if(Jr(t)!==t)throw Error(E(188))}function gk(t){var e=t.alternate;if(!e){if(e=Jr(t),e===null)throw Error(E(188));return e!==t?null:t}for(var n=t,r=e;;){var i=n.return;if(i===null)break;var s=i.alternate;if(s===null){if(r=i.return,r!==null){n=r;continue}break}if(i.child===s.child){for(s=i.child;s;){if(s===n)return ap(i),t;if(s===r)return ap(i),e;s=s.sibling}throw Error(E(188))}if(n.return!==r.return)n=i,r=s;else{for(var o=!1,l=i.child;l;){if(l===n){o=!0,n=i,r=s;break}if(l===r){o=!0,r=i,n=s;break}l=l.sibling}if(!o){for(l=s.child;l;){if(l===n){o=!0,n=s,r=i;break}if(l===r){o=!0,r=s,n=i;break}l=l.sibling}if(!o)throw Error(E(189))}}if(n.alternate!==r)throw Error(E(190))}if(n.tag!==3)throw Error(E(188));return n.stateNode.current===n?t:e}function M0(t){return t=gk(t),t!==null?I0(t):null}function I0(t){if(t.tag===5||t.tag===6)return t;for(t=t.child;t!==null;){var e=I0(t);if(e!==null)return e;t=t.sibling}return null}var A0=Mt.unstable_scheduleCallback,cp=Mt.unstable_cancelCallback,vk=Mt.unstable_shouldYield,yk=Mt.unstable_requestPaint,Pe=Mt.unstable_now,wk=Mt.unstable_getCurrentPriorityLevel,xf=Mt.unstable_ImmediatePriority,P0=Mt.unstable_UserBlockingPriority,$l=Mt.unstable_NormalPriority,Sk=Mt.unstable_LowPriority,E0=Mt.unstable_IdlePriority,Ba=null,wn=null;function kk(t){if(wn&&typeof wn.onCommitFiberRoot=="function")try{wn.onCommitFiberRoot(Ba,t,void 0,(t.current.flags&128)===128)}catch{}}var Qt=Math.clz32?Math.clz32:bk,xk=Math.log,Ck=Math.LN2;function bk(t){return t>>>=0,t===0?32:31-(xk(t)/Ck|0)|0}var Wo=64,Fo=4194304;function ks(t){switch(t&-t){case 1:return 1;case 2:return 2;case 4:return 4;case 8:return 8;case 16:return 16;case 32:return 32;case 64:case 128:case 256:case 512:case 1024:case 2048:case 4096:case 8192:case 16384:case 32768:case 65536:case 131072:case 262144:case 524288:case 1048576:case 2097152:return t&4194240;case 4194304:case 8388608:case 16777216:case 33554432:case 67108864:return t&130023424;case 134217728:return 134217728;case 268435456:return 268435456;case 536870912:return 536870912;case 1073741824:return 1073741824;default:return t}}function Gl(t,e){var n=t.pendingLanes;if(n===0)return 0;var r=0,i=t.suspendedLanes,s=t.pingedLanes,o=n&268435455;if(o!==0){var l=o&~i;l!==0?r=ks(l):(s&=o,s!==0&&(r=ks(s)))}else o=n&~i,o!==0?r=ks(o):s!==0&&(r=ks(s));if(r===0)return 0;if(e!==0&&e!==r&&!(e&i)&&(i=r&-r,s=e&-e,i>=s||i===16&&(s&4194240)!==0))return e;if(r&4&&(r|=n&16),e=t.entangledLanes,e!==0)for(t=t.entanglements,e&=r;0<e;)n=31-Qt(e),i=1<<n,r|=t[n],e&=~i;return r}function Tk(t,e){switch(t){case 1:case 2:case 4:return e+250;case 8:case 16:case 32:case 64:case 128:case 256:case 512:case 1024:case 2048:case 4096:case 8192:case 16384:case 32768:case 65536:case 131072:case 262144:case 524288:case 1048576:case 2097152:return e+5e3;case 4194304:case 8388608:case 16777216:case 33554432:case 67108864:return-1;case 134217728:case 268435456:case 536870912:case 1073741824:return-1;default:return-1}}function Rk(t,e){for(var n=t.suspendedLanes,r=t.pingedLanes,i=t.expirationTimes,s=t.pendingLanes;0<s;){var o=31-Qt(s),l=1<<o,a=i[o];a===-1?(!(l&n)||l&r)&&(i[o]=Tk(l,e)):a<=e&&(t.expiredLanes|=l),s&=~l}}function qu(t){return t=t.pendingLanes&-1073741825,t!==0?t:t&1073741824?1073741824:0}function L0(){var t=Wo;return Wo<<=1,!(Wo&4194240)&&(Wo=64),t}function Cc(t){for(var e=[],n=0;31>n;n++)e.push(t);return e}function Co(t,e,n){t.pendingLanes|=e,e!==536870912&&(t.suspendedLanes=0,t.pingedLanes=0),t=t.eventTimes,e=31-Qt(e),t[e]=n}function Mk(t,e){var n=t.pendingLanes&~e;t.pendingLanes=e,t.suspendedLanes=0,t.pingedLanes=0,t.expiredLanes&=e,t.mutableReadLanes&=e,t.entangledLanes&=e,e=t.entanglements;var r=t.eventTimes;for(t=t.expirationTimes;0<n;){var i=31-Qt(n),s=1<<i;e[i]=0,r[i]=-1,t[i]=-1,n&=~s}}function Cf(t,e){var n=t.entangledLanes|=e;for(t=t.entanglements;n;){var r=31-Qt(n),i=1<<r;i&e|t[r]&e&&(t[r]|=e),n&=~i}}var oe=0;function D0(t){return t&=-t,1<t?4<t?t&268435455?16:536870912:4:1}var O0,bf,N0,B0,_0,Wu=!1,zo=[],Zn=null,er=null,tr=null,$s=new Map,Gs=new Map,Hn=[],Ik="mousedown mouseup touchcancel touchend touchstart auxclick dblclick pointercancel pointerdown pointerup dragend dragstart drop compositionend compositionstart keydown keypress keyup input textInput copy cut paste click change contextmenu reset submit".split(" ");function up(t,e){switch(t){case"focusin":case"focusout":Zn=null;break;case"dragenter":case"dragleave":er=null;break;case"mouseover":case"mouseout":tr=null;break;case"pointerover":case"pointerout":$s.delete(e.pointerId);break;case"gotpointercapture":case"lostpointercapture":Gs.delete(e.pointerId)}}function ls(t,e,n,r,i,s){return t===null||t.nativeEvent!==s?(t={blockedOn:e,domEventName:n,eventSystemFlags:r,nativeEvent:s,targetContainers:[i]},e!==null&&(e=To(e),e!==null&&bf(e)),t):(t.eventSystemFlags|=r,e=t.targetContainers,i!==null&&e.indexOf(i)===-1&&e.push(i),t)}function Ak(t,e,n,r,i){switch(e){case"focusin":return Zn=ls(Zn,t,e,n,r,i),!0;case"dragenter":return er=ls(er,t,e,n,r,i),!0;case"mouseover":return tr=ls(tr,t,e,n,r,i),!0;case"pointerover":var s=i.pointerId;return $s.set(s,ls($s.get(s)||null,t,e,n,r,i)),!0;case"gotpointercapture":return s=i.pointerId,Gs.set(s,ls(Gs.get(s)||null,t,e,n,r,i)),!0}return!1}function q0(t){var e=Pr(t.target);if(e!==null){var n=Jr(e);if(n!==null){if(e=n.tag,e===13){if(e=R0(n),e!==null){t.blockedOn=e,_0(t.priority,function(){N0(n)});return}}else if(e===3&&n.stateNode.current.memoizedState.isDehydrated){t.blockedOn=n.tag===3?n.stateNode.containerInfo:null;return}}}t.blockedOn=null}function Cl(t){if(t.blockedOn!==null)return!1;for(var e=t.targetContainers;0<e.length;){var n=Fu(t.domEventName,t.eventSystemFlags,e[0],t.nativeEvent);if(n===null){n=t.nativeEvent;var r=new n.constructor(n.type,n);Ou=r,n.target.dispatchEvent(r),Ou=null}else return e=To(n),e!==null&&bf(e),t.blockedOn=n,!1;e.shift()}return!0}function hp(t,e,n){Cl(t)&&n.delete(e)}function Pk(){Wu=!1,Zn!==null&&Cl(Zn)&&(Zn=null),er!==null&&Cl(er)&&(er=null),tr!==null&&Cl(tr)&&(tr=null),$s.forEach(hp),Gs.forEach(hp)}function as(t,e){t.blockedOn===e&&(t.blockedOn=null,Wu||(Wu=!0,Mt.unstable_scheduleCallback(Mt.unstable_NormalPriority,Pk)))}function Ks(t){function e(i){return as(i,t)}if(0<zo.length){as(zo[0],t);for(var n=1;n<zo.length;n++){var r=zo[n];r.blockedOn===t&&(r.blockedOn=null)}}for(Zn!==null&&as(Zn,t),er!==null&&as(er,t),tr!==null&&as(tr,t),$s.forEach(e),Gs.forEach(e),n=0;n<Hn.length;n++)r=Hn[n],r.blockedOn===t&&(r.blockedOn=null);for(;0<Hn.length&&(n=Hn[0],n.blockedOn===null);)q0(n),n.blockedOn===null&&Hn.shift()}var Ti=Bn.ReactCurrentBatchConfig,Kl=!0;function Ek(t,e,n,r){var i=oe,s=Ti.transition;Ti.transition=null;try{oe=1,Tf(t,e,n,r)}finally{oe=i,Ti.transition=s}}function Lk(t,e,n,r){var i=oe,s=Ti.transition;Ti.transition=null;try{oe=4,Tf(t,e,n,r)}finally{oe=i,Ti.transition=s}}function Tf(t,e,n,r){if(Kl){var i=Fu(t,e,n,r);if(i===null)Dc(t,e,r,Ql,n),up(t,r);else if(Ak(i,t,e,n,r))r.stopPropagation();else if(up(t,r),e&4&&-1<Ik.indexOf(t)){for(;i!==null;){var s=To(i);if(s!==null&&O0(s),s=Fu(t,e,n,r),s===null&&Dc(t,e,r,Ql,n),s===i)break;i=s}i!==null&&r.stopPropagation()}else Dc(t,e,r,null,n)}}var Ql=null;function Fu(t,e,n,r){if(Ql=null,t=kf(r),t=Pr(t),t!==null)if(e=Jr(t),e===null)t=null;else if(n=e.tag,n===13){if(t=R0(e),t!==null)return t;t=null}else if(n===3){if(e.stateNode.current.memoizedState.isDehydrated)return e.tag===3?e.stateNode.containerInfo:null;t=null}else e!==t&&(t=null);return Ql=t,null}function W0(t){switch(t){case"cancel":case"click":case"close":case"contextmenu":case"copy":case"cut":case"auxclick":case"dblclick":case"dragend":case"dragstart":case"drop":case"focusin":case"focusout":case"input":case"invalid":case"keydown":case"keypress":case"keyup":case"mousedown":case"mouseup":case"paste":case"pause":case"play":case"pointercancel":case"pointerdown":case"pointerup":case"ratechange":case"reset":case"resize":case"seeked":case"submit":case"touchcancel":case"touchend":case"touchstart":case"volumechange":case"change":case"selectionchange":case"textInput":case"compositionstart":case"compositionend":case"compositionupdate":case"beforeblur":case"afterblur":case"beforeinput":case"blur":case"fullscreenchange":case"focus":case"hashchange":case"popstate":case"select":case"selectstart":return 1;case"drag":case"dragenter":case"dragexit":case"dragleave":case"dragover":case"mousemove":case"mouseout":case"mouseover":case"pointermove":case"pointerout":case"pointerover":case"scroll":case"toggle":case"touchmove":case"wheel":case"mouseenter":case"mouseleave":case"pointerenter":case"pointerleave":return 4;case"message":switch(wk()){case xf:return 1;case P0:return 4;case $l:case Sk:return 16;case E0:return 536870912;default:return 16}default:return 16}}var Qn=null,Rf=null,bl=null;function F0(){if(bl)return bl;var t,e=Rf,n=e.length,r,i="value"in Qn?Qn.value:Qn.textContent,s=i.length;for(t=0;t<n&&e[t]===i[t];t++);var o=n-t;for(r=1;r<=o&&e[n-r]===i[s-r];r++);return bl=i.slice(t,1<r?1-r:void 0)}function Tl(t){var e=t.keyCode;return"charCode"in t?(t=t.charCode,t===0&&e===13&&(t=13)):t=e,t===10&&(t=13),32<=t||t===13?t:0}function jo(){return!0}function fp(){return!1}function At(t){function e(n,r,i,s,o){this._reactName=n,this._targetInst=i,this.type=r,this.nativeEvent=s,this.target=o,this.currentTarget=null;for(var l in t)t.hasOwnProperty(l)&&(n=t[l],this[l]=n?n(s):s[l]);return this.isDefaultPrevented=(s.defaultPrevented!=null?s.defaultPrevented:s.returnValue===!1)?jo:fp,this.isPropagationStopped=fp,this}return be(e.prototype,{preventDefault:function(){this.defaultPrevented=!0;var n=this.nativeEvent;n&&(n.preventDefault?n.preventDefault():typeof n.returnValue!="unknown"&&(n.returnValue=!1),this.isDefaultPrevented=jo)},stopPropagation:function(){var n=this.nativeEvent;n&&(n.stopPropagation?n.stopPropagation():typeof n.cancelBubble!="unknown"&&(n.cancelBubble=!0),this.isPropagationStopped=jo)},persist:function(){},isPersistent:jo}),e}var Ji={eventPhase:0,bubbles:0,cancelable:0,timeStamp:function(t){return t.timeStamp||Date.now()},defaultPrevented:0,isTrusted:0},Mf=At(Ji),bo=be({},Ji,{view:0,detail:0}),Dk=At(bo),bc,Tc,cs,_a=be({},bo,{screenX:0,screenY:0,clientX:0,clientY:0,pageX:0,pageY:0,ctrlKey:0,shiftKey:0,altKey:0,metaKey:0,getModifierState:If,button:0,buttons:0,relatedTarget:function(t){return t.relatedTarget===void 0?t.fromElement===t.srcElement?t.toElement:t.fromElement:t.relatedTarget},movementX:function(t){return"movementX"in t?t.movementX:(t!==cs&&(cs&&t.type==="mousemove"?(bc=t.screenX-cs.screenX,Tc=t.screenY-cs.screenY):Tc=bc=0,cs=t),bc)},movementY:function(t){return"movementY"in t?t.movementY:Tc}}),dp=At(_a),Ok=be({},_a,{dataTransfer:0}),Nk=At(Ok),Bk=be({},bo,{relatedTarget:0}),Rc=At(Bk),_k=be({},Ji,{animationName:0,elapsedTime:0,pseudoElement:0}),qk=At(_k),Wk=be({},Ji,{clipboardData:function(t){return"clipboardData"in t?t.clipboardData:window.clipboardData}}),Fk=At(Wk),zk=be({},Ji,{data:0}),pp=At(zk),jk={Esc:"Escape",Spacebar:" ",Left:"ArrowLeft",Up:"ArrowUp",Right:"ArrowRight",Down:"ArrowDown",Del:"Delete",Win:"OS",Menu:"ContextMenu",Apps:"ContextMenu",Scroll:"ScrollLock",MozPrintableKey:"Unidentified"},Vk={8:"Backspace",9:"Tab",12:"Clear",13:"Enter",16:"Shift",17:"Control",18:"Alt",19:"Pause",20:"CapsLock",27:"Escape",32:" ",33:"PageUp",34:"PageDown",35:"End",36:"Home",37:"ArrowLeft",38:"ArrowUp",39:"ArrowRight",40:"ArrowDown",45:"Insert",46:"Delete",112:"F1",113:"F2",114:"F3",115:"F4",116:"F5",117:"F6",118:"F7",119:"F8",120:"F9",121:"F10",122:"F11",123:"F12",144:"NumLock",145:"ScrollLock",224:"Meta"},Hk={Alt:"altKey",Control:"ctrlKey",Meta:"metaKey",Shift:"shiftKey"};function Uk(t){var e=this.nativeEvent;return e.getModifierState?e.getModifierState(t):(t=Hk[t])?!!e[t]:!1}function If(){return Uk}var $k=be({},bo,{key:function(t){if(t.key){var e=jk[t.key]||t.key;if(e!=="Unidentified")return e}return t.type==="keypress"?(t=Tl(t),t===13?"Enter":String.fromCharCode(t)):t.type==="keydown"||t.type==="keyup"?Vk[t.keyCode]||"Unidentified":""},code:0,location:0,ctrlKey:0,shiftKey:0,altKey:0,metaKey:0,repeat:0,locale:0,getModifierState:If,charCode:function(t){return t.type==="keypress"?Tl(t):0},keyCode:function(t){return t.type==="keydown"||t.type==="keyup"?t.keyCode:0},which:function(t){return t.type==="keypress"?Tl(t):t.type==="keydown"||t.type==="keyup"?t.keyCode:0}}),Gk=At($k),Kk=be({},_a,{pointerId:0,width:0,height:0,pressure:0,tangentialPressure:0,tiltX:0,tiltY:0,twist:0,pointerType:0,isPrimary:0}),mp=At(Kk),Qk=be({},bo,{touches:0,targetTouches:0,changedTouches:0,altKey:0,metaKey:0,ctrlKey:0,shiftKey:0,getModifierState:If}),Yk=At(Qk),Xk=be({},Ji,{propertyName:0,elapsedTime:0,pseudoElement:0}),Jk=At(Xk),Zk=be({},_a,{deltaX:function(t){return"deltaX"in t?t.deltaX:"wheelDeltaX"in t?-t.wheelDeltaX:0},deltaY:function(t){return"deltaY"in t?t.deltaY:"wheelDeltaY"in t?-t.wheelDeltaY:"wheelDelta"in t?-t.wheelDelta:0},deltaZ:0,deltaMode:0}),ex=At(Zk),tx=[9,13,27,32],Af=En&&"CompositionEvent"in window,Is=null;En&&"documentMode"in document&&(Is=document.documentMode);var nx=En&&"TextEvent"in window&&!Is,z0=En&&(!Af||Is&&8<Is&&11>=Is),gp=" ",vp=!1;function j0(t,e){switch(t){case"keyup":return tx.indexOf(e.keyCode)!==-1;case"keydown":return e.keyCode!==229;case"keypress":case"mousedown":case"focusout":return!0;default:return!1}}function V0(t){return t=t.detail,typeof t=="object"&&"data"in t?t.data:null}var ai=!1;function rx(t,e){switch(t){case"compositionend":return V0(e);case"keypress":return e.which!==32?null:(vp=!0,gp);case"textInput":return t=e.data,t===gp&&vp?null:t;default:return null}}function ix(t,e){if(ai)return t==="compositionend"||!Af&&j0(t,e)?(t=F0(),bl=Rf=Qn=null,ai=!1,t):null;switch(t){case"paste":return null;case"keypress":if(!(e.ctrlKey||e.altKey||e.metaKey)||e.ctrlKey&&e.altKey){if(e.char&&1<e.char.length)return e.char;if(e.which)return String.fromCharCode(e.which)}return null;case"compositionend":return z0&&e.locale!=="ko"?null:e.data;default:return null}}var sx={color:!0,date:!0,datetime:!0,"datetime-local":!0,email:!0,month:!0,number:!0,password:!0,range:!0,search:!0,tel:!0,text:!0,time:!0,url:!0,week:!0};function yp(t){var e=t&&t.nodeName&&t.nodeName.toLowerCase();return e==="input"?!!sx[t.type]:e==="textarea"}function H0(t,e,n,r){k0(r),e=Yl(e,"onChange"),0<e.length&&(n=new Mf("onChange","change",null,n,r),t.push({event:n,listeners:e}))}var As=null,Qs=null;function ox(t){tv(t,0)}function qa(t){var e=hi(t);if(p0(e))return t}function lx(t,e){if(t==="change")return e}var U0=!1;if(En){var Mc;if(En){var Ic="oninput"in document;if(!Ic){var wp=document.createElement("div");wp.setAttribute("oninput","return;"),Ic=typeof wp.oninput=="function"}Mc=Ic}else Mc=!1;U0=Mc&&(!document.documentMode||9<document.documentMode)}function Sp(){As&&(As.detachEvent("onpropertychange",$0),Qs=As=null)}function $0(t){if(t.propertyName==="value"&&qa(Qs)){var e=[];H0(e,Qs,t,kf(t)),T0(ox,e)}}function ax(t,e,n){t==="focusin"?(Sp(),As=e,Qs=n,As.attachEvent("onpropertychange",$0)):t==="focusout"&&Sp()}function cx(t){if(t==="selectionchange"||t==="keyup"||t==="keydown")return qa(Qs)}function ux(t,e){if(t==="click")return qa(e)}function hx(t,e){if(t==="input"||t==="change")return qa(e)}function fx(t,e){return t===e&&(t!==0||1/t===1/e)||t!==t&&e!==e}var Xt=typeof Object.is=="function"?Object.is:fx;function Ys(t,e){if(Xt(t,e))return!0;if(typeof t!="object"||t===null||typeof e!="object"||e===null)return!1;var n=Object.keys(t),r=Object.keys(e);if(n.length!==r.length)return!1;for(r=0;r<n.length;r++){var i=n[r];if(!xu.call(e,i)||!Xt(t[i],e[i]))return!1}return!0}function kp(t){for(;t&&t.firstChild;)t=t.firstChild;return t}function xp(t,e){var n=kp(t);t=0;for(var r;n;){if(n.nodeType===3){if(r=t+n.textContent.length,t<=e&&r>=e)return{node:n,offset:e-t};t=r}e:{for(;n;){if(n.nextSibling){n=n.nextSibling;break e}n=n.parentNode}n=void 0}n=kp(n)}}function G0(t,e){return t&&e?t===e?!0:t&&t.nodeType===3?!1:e&&e.nodeType===3?G0(t,e.parentNode):"contains"in t?t.contains(e):t.compareDocumentPosition?!!(t.compareDocumentPosition(e)&16):!1:!1}function K0(){for(var t=window,e=Vl();e instanceof t.HTMLIFrameElement;){try{var n=typeof e.contentWindow.location.href=="string"}catch{n=!1}if(n)t=e.contentWindow;else break;e=Vl(t.document)}return e}function Pf(t){var e=t&&t.nodeName&&t.nodeName.toLowerCase();return e&&(e==="input"&&(t.type==="text"||t.type==="search"||t.type==="tel"||t.type==="url"||t.type==="password")||e==="textarea"||t.contentEditable==="true")}function dx(t){var e=K0(),n=t.focusedElem,r=t.selectionRange;if(e!==n&&n&&n.ownerDocument&&G0(n.ownerDocument.documentElement,n)){if(r!==null&&Pf(n)){if(e=r.start,t=r.end,t===void 0&&(t=e),"selectionStart"in n)n.selectionStart=e,n.selectionEnd=Math.min(t,n.value.length);else if(t=(e=n.ownerDocument||document)&&e.defaultView||window,t.getSelection){t=t.getSelection();var i=n.textContent.length,s=Math.min(r.start,i);r=r.end===void 0?s:Math.min(r.end,i),!t.extend&&s>r&&(i=r,r=s,s=i),i=xp(n,s);var o=xp(n,r);i&&o&&(t.rangeCount!==1||t.anchorNode!==i.node||t.anchorOffset!==i.offset||t.focusNode!==o.node||t.focusOffset!==o.offset)&&(e=e.createRange(),e.setStart(i.node,i.offset),t.removeAllRanges(),s>r?(t.addRange(e),t.extend(o.node,o.offset)):(e.setEnd(o.node,o.offset),t.addRange(e)))}}for(e=[],t=n;t=t.parentNode;)t.nodeType===1&&e.push({element:t,left:t.scrollLeft,top:t.scrollTop});for(typeof n.focus=="function"&&n.focus(),n=0;n<e.length;n++)t=e[n],t.element.scrollLeft=t.left,t.element.scrollTop=t.top}}var px=En&&"documentMode"in document&&11>=document.documentMode,ci=null,zu=null,Ps=null,ju=!1;function Cp(t,e,n){var r=n.window===n?n.document:n.nodeType===9?n:n.ownerDocument;ju||ci==null||ci!==Vl(r)||(r=ci,"selectionStart"in r&&Pf(r)?r={start:r.selectionStart,end:r.selectionEnd}:(r=(r.ownerDocument&&r.ownerDocument.defaultView||window).getSelection(),r={anchorNode:r.anchorNode,anchorOffset:r.anchorOffset,focusNode:r.focusNode,focusOffset:r.focusOffset}),Ps&&Ys(Ps,r)||(Ps=r,r=Yl(zu,"onSelect"),0<r.length&&(e=new Mf("onSelect","select",null,e,n),t.push({event:e,listeners:r}),e.target=ci)))}function Vo(t,e){var n={};return n[t.toLowerCase()]=e.toLowerCase(),n["Webkit"+t]="webkit"+e,n["Moz"+t]="moz"+e,n}var ui={animationend:Vo("Animation","AnimationEnd"),animationiteration:Vo("Animation","AnimationIteration"),animationstart:Vo("Animation","AnimationStart"),transitionend:Vo("Transition","TransitionEnd")},Ac={},Q0={};En&&(Q0=document.createElement("div").style,"AnimationEvent"in window||(delete ui.animationend.animation,delete ui.animationiteration.animation,delete ui.animationstart.animation),"TransitionEvent"in window||delete ui.transitionend.transition);function Wa(t){if(Ac[t])return Ac[t];if(!ui[t])return t;var e=ui[t],n;for(n in e)if(e.hasOwnProperty(n)&&n in Q0)return Ac[t]=e[n];return t}var Y0=Wa("animationend"),X0=Wa("animationiteration"),J0=Wa("animationstart"),Z0=Wa("transitionend"),ev=new Map,bp="abort auxClick cancel canPlay canPlayThrough click close contextMenu copy cut drag dragEnd dragEnter dragExit dragLeave dragOver dragStart drop durationChange emptied encrypted ended error gotPointerCapture input invalid keyDown keyPress keyUp load loadedData loadedMetadata loadStart lostPointerCapture mouseDown mouseMove mouseOut mouseOver mouseUp paste pause play playing pointerCancel pointerDown pointerMove pointerOut pointerOver pointerUp progress rateChange reset resize seeked seeking stalled submit suspend timeUpdate touchCancel touchEnd touchStart volumeChange scroll toggle touchMove waiting wheel".split(" ");function vr(t,e){ev.set(t,e),Xr(e,[t])}for(var Pc=0;Pc<bp.length;Pc++){var Ec=bp[Pc],mx=Ec.toLowerCase(),gx=Ec[0].toUpperCase()+Ec.slice(1);vr(mx,"on"+gx)}vr(Y0,"onAnimationEnd");vr(X0,"onAnimationIteration");vr(J0,"onAnimationStart");vr("dblclick","onDoubleClick");vr("focusin","onFocus");vr("focusout","onBlur");vr(Z0,"onTransitionEnd");Ni("onMouseEnter",["mouseout","mouseover"]);Ni("onMouseLeave",["mouseout","mouseover"]);Ni("onPointerEnter",["pointerout","pointerover"]);Ni("onPointerLeave",["pointerout","pointerover"]);Xr("onChange","change click focusin focusout input keydown keyup selectionchange".split(" "));Xr("onSelect","focusout contextmenu dragend focusin keydown keyup mousedown mouseup selectionchange".split(" "));Xr("onBeforeInput",["compositionend","keypress","textInput","paste"]);Xr("onCompositionEnd","compositionend focusout keydown keypress keyup mousedown".split(" "));Xr("onCompositionStart","compositionstart focusout keydown keypress keyup mousedown".split(" "));Xr("onCompositionUpdate","compositionupdate focusout keydown keypress keyup mousedown".split(" "));var xs="abort canplay canplaythrough durationchange emptied encrypted ended error loadeddata loadedmetadata loadstart pause play playing progress ratechange resize seeked seeking stalled suspend timeupdate volumechange waiting".split(" "),vx=new Set("cancel close invalid load scroll toggle".split(" ").concat(xs));function Tp(t,e,n){var r=t.type||"unknown-event";t.currentTarget=n,mk(r,e,void 0,t),t.currentTarget=null}function tv(t,e){e=(e&4)!==0;for(var n=0;n<t.length;n++){var r=t[n],i=r.event;r=r.listeners;e:{var s=void 0;if(e)for(var o=r.length-1;0<=o;o--){var l=r[o],a=l.instance,c=l.currentTarget;if(l=l.listener,a!==s&&i.isPropagationStopped())break e;Tp(i,l,c),s=a}else for(o=0;o<r.length;o++){if(l=r[o],a=l.instance,c=l.currentTarget,l=l.listener,a!==s&&i.isPropagationStopped())break e;Tp(i,l,c),s=a}}}if(Ul)throw t=_u,Ul=!1,_u=null,t}function ge(t,e){var n=e[Gu];n===void 0&&(n=e[Gu]=new Set);var r=t+"__bubble";n.has(r)||(nv(e,t,2,!1),n.add(r))}function Lc(t,e,n){var r=0;e&&(r|=4),nv(n,t,r,e)}var Ho="_reactListening"+Math.random().toString(36).slice(2);function Xs(t){if(!t[Ho]){t[Ho]=!0,c0.forEach(function(n){n!=="selectionchange"&&(vx.has(n)||Lc(n,!1,t),Lc(n,!0,t))});var e=t.nodeType===9?t:t.ownerDocument;e===null||e[Ho]||(e[Ho]=!0,Lc("selectionchange",!1,e))}}function nv(t,e,n,r){switch(W0(e)){case 1:var i=Ek;break;case 4:i=Lk;break;default:i=Tf}n=i.bind(null,e,n,t),i=void 0,!Bu||e!=="touchstart"&&e!=="touchmove"&&e!=="wheel"||(i=!0),r?i!==void 0?t.addEventListener(e,n,{capture:!0,passive:i}):t.addEventListener(e,n,!0):i!==void 0?t.addEventListener(e,n,{passive:i}):t.addEventListener(e,n,!1)}function Dc(t,e,n,r,i){var s=r;if(!(e&1)&&!(e&2)&&r!==null)e:for(;;){if(r===null)return;var o=r.tag;if(o===3||o===4){var l=r.stateNode.containerInfo;if(l===i||l.nodeType===8&&l.parentNode===i)break;if(o===4)for(o=r.return;o!==null;){var a=o.tag;if((a===3||a===4)&&(a=o.stateNode.containerInfo,a===i||a.nodeType===8&&a.parentNode===i))return;o=o.return}for(;l!==null;){if(o=Pr(l),o===null)return;if(a=o.tag,a===5||a===6){r=s=o;continue e}l=l.parentNode}}r=r.return}T0(function(){var c=s,u=kf(n),h=[];e:{var f=ev.get(t);if(f!==void 0){var d=Mf,p=t;switch(t){case"keypress":if(Tl(n)===0)break e;case"keydown":case"keyup":d=Gk;break;case"focusin":p="focus",d=Rc;break;case"focusout":p="blur",d=Rc;break;case"beforeblur":case"afterblur":d=Rc;break;case"click":if(n.button===2)break e;case"auxclick":case"dblclick":case"mousedown":case"mousemove":case"mouseup":case"mouseout":case"mouseover":case"contextmenu":d=dp;break;case"drag":case"dragend":case"dragenter":case"dragexit":case"dragleave":case"dragover":case"dragstart":case"drop":d=Nk;break;case"touchcancel":case"touchend":case"touchmove":case"touchstart":d=Yk;break;case Y0:case X0:case J0:d=qk;break;case Z0:d=Jk;break;case"scroll":d=Dk;break;case"wheel":d=ex;break;case"copy":case"cut":case"paste":d=Fk;break;case"gotpointercapture":case"lostpointercapture":case"pointercancel":case"pointerdown":case"pointermove":case"pointerout":case"pointerover":case"pointerup":d=mp}var m=(e&4)!==0,w=!m&&t==="scroll",g=m?f!==null?f+"Capture":null:f;m=[];for(var v=c,y;v!==null;){y=v;var x=y.stateNode;if(y.tag===5&&x!==null&&(y=x,g!==null&&(x=Us(v,g),x!=null&&m.push(Js(v,x,y)))),w)break;v=v.return}0<m.length&&(f=new d(f,p,null,n,u),h.push({event:f,listeners:m}))}}if(!(e&7)){e:{if(f=t==="mouseover"||t==="pointerover",d=t==="mouseout"||t==="pointerout",f&&n!==Ou&&(p=n.relatedTarget||n.fromElement)&&(Pr(p)||p[Ln]))break e;if((d||f)&&(f=u.window===u?u:(f=u.ownerDocument)?f.defaultView||f.parentWindow:window,d?(p=n.relatedTarget||n.toElement,d=c,p=p?Pr(p):null,p!==null&&(w=Jr(p),p!==w||p.tag!==5&&p.tag!==6)&&(p=null)):(d=null,p=c),d!==p)){if(m=dp,x="onMouseLeave",g="onMouseEnter",v="mouse",(t==="pointerout"||t==="pointerover")&&(m=mp,x="onPointerLeave",g="onPointerEnter",v="pointer"),w=d==null?f:hi(d),y=p==null?f:hi(p),f=new m(x,v+"leave",d,n,u),f.target=w,f.relatedTarget=y,x=null,Pr(u)===c&&(m=new m(g,v+"enter",p,n,u),m.target=y,m.relatedTarget=w,x=m),w=x,d&&p)t:{for(m=d,g=p,v=0,y=m;y;y=ti(y))v++;for(y=0,x=g;x;x=ti(x))y++;for(;0<v-y;)m=ti(m),v--;for(;0<y-v;)g=ti(g),y--;for(;v--;){if(m===g||g!==null&&m===g.alternate)break t;m=ti(m),g=ti(g)}m=null}else m=null;d!==null&&Rp(h,f,d,m,!1),p!==null&&w!==null&&Rp(h,w,p,m,!0)}}e:{if(f=c?hi(c):window,d=f.nodeName&&f.nodeName.toLowerCase(),d==="select"||d==="input"&&f.type==="file")var k=lx;else if(yp(f))if(U0)k=hx;else{k=cx;var C=ax}else(d=f.nodeName)&&d.toLowerCase()==="input"&&(f.type==="checkbox"||f.type==="radio")&&(k=ux);if(k&&(k=k(t,c))){H0(h,k,n,u);break e}C&&C(t,f,c),t==="focusout"&&(C=f._wrapperState)&&C.controlled&&f.type==="number"&&Au(f,"number",f.value)}switch(C=c?hi(c):window,t){case"focusin":(yp(C)||C.contentEditable==="true")&&(ci=C,zu=c,Ps=null);break;case"focusout":Ps=zu=ci=null;break;case"mousedown":ju=!0;break;case"contextmenu":case"mouseup":case"dragend":ju=!1,Cp(h,n,u);break;case"selectionchange":if(px)break;case"keydown":case"keyup":Cp(h,n,u)}var b;if(Af)e:{switch(t){case"compositionstart":var I="onCompositionStart";break e;case"compositionend":I="onCompositionEnd";break e;case"compositionupdate":I="onCompositionUpdate";break e}I=void 0}else ai?j0(t,n)&&(I="onCompositionEnd"):t==="keydown"&&n.keyCode===229&&(I="onCompositionStart");I&&(z0&&n.locale!=="ko"&&(ai||I!=="onCompositionStart"?I==="onCompositionEnd"&&ai&&(b=F0()):(Qn=u,Rf="value"in Qn?Qn.value:Qn.textContent,ai=!0)),C=Yl(c,I),0<C.length&&(I=new pp(I,t,null,n,u),h.push({event:I,listeners:C}),b?I.data=b:(b=V0(n),b!==null&&(I.data=b)))),(b=nx?rx(t,n):ix(t,n))&&(c=Yl(c,"onBeforeInput"),0<c.length&&(u=new pp("onBeforeInput","beforeinput",null,n,u),h.push({event:u,listeners:c}),u.data=b))}tv(h,e)})}function Js(t,e,n){return{instance:t,listener:e,currentTarget:n}}function Yl(t,e){for(var n=e+"Capture",r=[];t!==null;){var i=t,s=i.stateNode;i.tag===5&&s!==null&&(i=s,s=Us(t,n),s!=null&&r.unshift(Js(t,s,i)),s=Us(t,e),s!=null&&r.push(Js(t,s,i))),t=t.return}return r}function ti(t){if(t===null)return null;do t=t.return;while(t&&t.tag!==5);return t||null}function Rp(t,e,n,r,i){for(var s=e._reactName,o=[];n!==null&&n!==r;){var l=n,a=l.alternate,c=l.stateNode;if(a!==null&&a===r)break;l.tag===5&&c!==null&&(l=c,i?(a=Us(n,s),a!=null&&o.unshift(Js(n,a,l))):i||(a=Us(n,s),a!=null&&o.push(Js(n,a,l)))),n=n.return}o.length!==0&&t.push({event:e,listeners:o})}var yx=/\r\n?/g,wx=/\u0000|\uFFFD/g;function Mp(t){return(typeof t=="string"?t:""+t).replace(yx,`
`).replace(wx,"")}function Uo(t,e,n){if(e=Mp(e),Mp(t)!==e&&n)throw Error(E(425))}function Xl(){}var Vu=null,Hu=null;function Uu(t,e){return t==="textarea"||t==="noscript"||typeof e.children=="string"||typeof e.children=="number"||typeof e.dangerouslySetInnerHTML=="object"&&e.dangerouslySetInnerHTML!==null&&e.dangerouslySetInnerHTML.__html!=null}var $u=typeof setTimeout=="function"?setTimeout:void 0,Sx=typeof clearTimeout=="function"?clearTimeout:void 0,Ip=typeof Promise=="function"?Promise:void 0,kx=typeof queueMicrotask=="function"?queueMicrotask:typeof Ip<"u"?function(t){return Ip.resolve(null).then(t).catch(xx)}:$u;function xx(t){setTimeout(function(){throw t})}function Oc(t,e){var n=e,r=0;do{var i=n.nextSibling;if(t.removeChild(n),i&&i.nodeType===8)if(n=i.data,n==="/$"){if(r===0){t.removeChild(i),Ks(e);return}r--}else n!=="$"&&n!=="$?"&&n!=="$!"||r++;n=i}while(n);Ks(e)}function nr(t){for(;t!=null;t=t.nextSibling){var e=t.nodeType;if(e===1||e===3)break;if(e===8){if(e=t.data,e==="$"||e==="$!"||e==="$?")break;if(e==="/$")return null}}return t}function Ap(t){t=t.previousSibling;for(var e=0;t;){if(t.nodeType===8){var n=t.data;if(n==="$"||n==="$!"||n==="$?"){if(e===0)return t;e--}else n==="/$"&&e++}t=t.previousSibling}return null}var Zi=Math.random().toString(36).slice(2),un="__reactFiber$"+Zi,Zs="__reactProps$"+Zi,Ln="__reactContainer$"+Zi,Gu="__reactEvents$"+Zi,Cx="__reactListeners$"+Zi,bx="__reactHandles$"+Zi;function Pr(t){var e=t[un];if(e)return e;for(var n=t.parentNode;n;){if(e=n[Ln]||n[un]){if(n=e.alternate,e.child!==null||n!==null&&n.child!==null)for(t=Ap(t);t!==null;){if(n=t[un])return n;t=Ap(t)}return e}t=n,n=t.parentNode}return null}function To(t){return t=t[un]||t[Ln],!t||t.tag!==5&&t.tag!==6&&t.tag!==13&&t.tag!==3?null:t}function hi(t){if(t.tag===5||t.tag===6)return t.stateNode;throw Error(E(33))}function Fa(t){return t[Zs]||null}var Ku=[],fi=-1;function yr(t){return{current:t}}function ve(t){0>fi||(t.current=Ku[fi],Ku[fi]=null,fi--)}function fe(t,e){fi++,Ku[fi]=t.current,t.current=e}var cr={},Ze=yr(cr),vt=yr(!1),Fr=cr;function Bi(t,e){var n=t.type.contextTypes;if(!n)return cr;var r=t.stateNode;if(r&&r.__reactInternalMemoizedUnmaskedChildContext===e)return r.__reactInternalMemoizedMaskedChildContext;var i={},s;for(s in n)i[s]=e[s];return r&&(t=t.stateNode,t.__reactInternalMemoizedUnmaskedChildContext=e,t.__reactInternalMemoizedMaskedChildContext=i),i}function yt(t){return t=t.childContextTypes,t!=null}function Jl(){ve(vt),ve(Ze)}function Pp(t,e,n){if(Ze.current!==cr)throw Error(E(168));fe(Ze,e),fe(vt,n)}function rv(t,e,n){var r=t.stateNode;if(e=e.childContextTypes,typeof r.getChildContext!="function")return n;r=r.getChildContext();for(var i in r)if(!(i in e))throw Error(E(108,ak(t)||"Unknown",i));return be({},n,r)}function Zl(t){return t=(t=t.stateNode)&&t.__reactInternalMemoizedMergedChildContext||cr,Fr=Ze.current,fe(Ze,t),fe(vt,vt.current),!0}function Ep(t,e,n){var r=t.stateNode;if(!r)throw Error(E(169));n?(t=rv(t,e,Fr),r.__reactInternalMemoizedMergedChildContext=t,ve(vt),ve(Ze),fe(Ze,t)):ve(vt),fe(vt,n)}var Tn=null,za=!1,Nc=!1;function iv(t){Tn===null?Tn=[t]:Tn.push(t)}function Tx(t){za=!0,iv(t)}function wr(){if(!Nc&&Tn!==null){Nc=!0;var t=0,e=oe;try{var n=Tn;for(oe=1;t<n.length;t++){var r=n[t];do r=r(!0);while(r!==null)}Tn=null,za=!1}catch(i){throw Tn!==null&&(Tn=Tn.slice(t+1)),A0(xf,wr),i}finally{oe=e,Nc=!1}}return null}var di=[],pi=0,ea=null,ta=0,Pt=[],Et=0,zr=null,Mn=1,In="";function Tr(t,e){di[pi++]=ta,di[pi++]=ea,ea=t,ta=e}function sv(t,e,n){Pt[Et++]=Mn,Pt[Et++]=In,Pt[Et++]=zr,zr=t;var r=Mn;t=In;var i=32-Qt(r)-1;r&=~(1<<i),n+=1;var s=32-Qt(e)+i;if(30<s){var o=i-i%5;s=(r&(1<<o)-1).toString(32),r>>=o,i-=o,Mn=1<<32-Qt(e)+i|n<<i|r,In=s+t}else Mn=1<<s|n<<i|r,In=t}function Ef(t){t.return!==null&&(Tr(t,1),sv(t,1,0))}function Lf(t){for(;t===ea;)ea=di[--pi],di[pi]=null,ta=di[--pi],di[pi]=null;for(;t===zr;)zr=Pt[--Et],Pt[Et]=null,In=Pt[--Et],Pt[Et]=null,Mn=Pt[--Et],Pt[Et]=null}var Rt=null,bt=null,ye=!1,Ut=null;function ov(t,e){var n=Dt(5,null,null,0);n.elementType="DELETED",n.stateNode=e,n.return=t,e=t.deletions,e===null?(t.deletions=[n],t.flags|=16):e.push(n)}function Lp(t,e){switch(t.tag){case 5:var n=t.type;return e=e.nodeType!==1||n.toLowerCase()!==e.nodeName.toLowerCase()?null:e,e!==null?(t.stateNode=e,Rt=t,bt=nr(e.firstChild),!0):!1;case 6:return e=t.pendingProps===""||e.nodeType!==3?null:e,e!==null?(t.stateNode=e,Rt=t,bt=null,!0):!1;case 13:return e=e.nodeType!==8?null:e,e!==null?(n=zr!==null?{id:Mn,overflow:In}:null,t.memoizedState={dehydrated:e,treeContext:n,retryLane:1073741824},n=Dt(18,null,null,0),n.stateNode=e,n.return=t,t.child=n,Rt=t,bt=null,!0):!1;default:return!1}}function Qu(t){return(t.mode&1)!==0&&(t.flags&128)===0}function Yu(t){if(ye){var e=bt;if(e){var n=e;if(!Lp(t,e)){if(Qu(t))throw Error(E(418));e=nr(n.nextSibling);var r=Rt;e&&Lp(t,e)?ov(r,n):(t.flags=t.flags&-4097|2,ye=!1,Rt=t)}}else{if(Qu(t))throw Error(E(418));t.flags=t.flags&-4097|2,ye=!1,Rt=t}}}function Dp(t){for(t=t.return;t!==null&&t.tag!==5&&t.tag!==3&&t.tag!==13;)t=t.return;Rt=t}function $o(t){if(t!==Rt)return!1;if(!ye)return Dp(t),ye=!0,!1;var e;if((e=t.tag!==3)&&!(e=t.tag!==5)&&(e=t.type,e=e!=="head"&&e!=="body"&&!Uu(t.type,t.memoizedProps)),e&&(e=bt)){if(Qu(t))throw lv(),Error(E(418));for(;e;)ov(t,e),e=nr(e.nextSibling)}if(Dp(t),t.tag===13){if(t=t.memoizedState,t=t!==null?t.dehydrated:null,!t)throw Error(E(317));e:{for(t=t.nextSibling,e=0;t;){if(t.nodeType===8){var n=t.data;if(n==="/$"){if(e===0){bt=nr(t.nextSibling);break e}e--}else n!=="$"&&n!=="$!"&&n!=="$?"||e++}t=t.nextSibling}bt=null}}else bt=Rt?nr(t.stateNode.nextSibling):null;return!0}function lv(){for(var t=bt;t;)t=nr(t.nextSibling)}function _i(){bt=Rt=null,ye=!1}function Df(t){Ut===null?Ut=[t]:Ut.push(t)}var Rx=Bn.ReactCurrentBatchConfig;function us(t,e,n){if(t=n.ref,t!==null&&typeof t!="function"&&typeof t!="object"){if(n._owner){if(n=n._owner,n){if(n.tag!==1)throw Error(E(309));var r=n.stateNode}if(!r)throw Error(E(147,t));var i=r,s=""+t;return e!==null&&e.ref!==null&&typeof e.ref=="function"&&e.ref._stringRef===s?e.ref:(e=function(o){var l=i.refs;o===null?delete l[s]:l[s]=o},e._stringRef=s,e)}if(typeof t!="string")throw Error(E(284));if(!n._owner)throw Error(E(290,t))}return t}function Go(t,e){throw t=Object.prototype.toString.call(e),Error(E(31,t==="[object Object]"?"object with keys {"+Object.keys(e).join(", ")+"}":t))}function Op(t){var e=t._init;return e(t._payload)}function av(t){function e(g,v){if(t){var y=g.deletions;y===null?(g.deletions=[v],g.flags|=16):y.push(v)}}function n(g,v){if(!t)return null;for(;v!==null;)e(g,v),v=v.sibling;return null}function r(g,v){for(g=new Map;v!==null;)v.key!==null?g.set(v.key,v):g.set(v.index,v),v=v.sibling;return g}function i(g,v){return g=or(g,v),g.index=0,g.sibling=null,g}function s(g,v,y){return g.index=y,t?(y=g.alternate,y!==null?(y=y.index,y<v?(g.flags|=2,v):y):(g.flags|=2,v)):(g.flags|=1048576,v)}function o(g){return t&&g.alternate===null&&(g.flags|=2),g}function l(g,v,y,x){return v===null||v.tag!==6?(v=jc(y,g.mode,x),v.return=g,v):(v=i(v,y),v.return=g,v)}function a(g,v,y,x){var k=y.type;return k===li?u(g,v,y.props.children,x,y.key):v!==null&&(v.elementType===k||typeof k=="object"&&k!==null&&k.$$typeof===Fn&&Op(k)===v.type)?(x=i(v,y.props),x.ref=us(g,v,y),x.return=g,x):(x=Ll(y.type,y.key,y.props,null,g.mode,x),x.ref=us(g,v,y),x.return=g,x)}function c(g,v,y,x){return v===null||v.tag!==4||v.stateNode.containerInfo!==y.containerInfo||v.stateNode.implementation!==y.implementation?(v=Vc(y,g.mode,x),v.return=g,v):(v=i(v,y.children||[]),v.return=g,v)}function u(g,v,y,x,k){return v===null||v.tag!==7?(v=Br(y,g.mode,x,k),v.return=g,v):(v=i(v,y),v.return=g,v)}function h(g,v,y){if(typeof v=="string"&&v!==""||typeof v=="number")return v=jc(""+v,g.mode,y),v.return=g,v;if(typeof v=="object"&&v!==null){switch(v.$$typeof){case Bo:return y=Ll(v.type,v.key,v.props,null,g.mode,y),y.ref=us(g,null,v),y.return=g,y;case oi:return v=Vc(v,g.mode,y),v.return=g,v;case Fn:var x=v._init;return h(g,x(v._payload),y)}if(Ss(v)||ss(v))return v=Br(v,g.mode,y,null),v.return=g,v;Go(g,v)}return null}function f(g,v,y,x){var k=v!==null?v.key:null;if(typeof y=="string"&&y!==""||typeof y=="number")return k!==null?null:l(g,v,""+y,x);if(typeof y=="object"&&y!==null){switch(y.$$typeof){case Bo:return y.key===k?a(g,v,y,x):null;case oi:return y.key===k?c(g,v,y,x):null;case Fn:return k=y._init,f(g,v,k(y._payload),x)}if(Ss(y)||ss(y))return k!==null?null:u(g,v,y,x,null);Go(g,y)}return null}function d(g,v,y,x,k){if(typeof x=="string"&&x!==""||typeof x=="number")return g=g.get(y)||null,l(v,g,""+x,k);if(typeof x=="object"&&x!==null){switch(x.$$typeof){case Bo:return g=g.get(x.key===null?y:x.key)||null,a(v,g,x,k);case oi:return g=g.get(x.key===null?y:x.key)||null,c(v,g,x,k);case Fn:var C=x._init;return d(g,v,y,C(x._payload),k)}if(Ss(x)||ss(x))return g=g.get(y)||null,u(v,g,x,k,null);Go(v,x)}return null}function p(g,v,y,x){for(var k=null,C=null,b=v,I=v=0,W=null;b!==null&&I<y.length;I++){b.index>I?(W=b,b=null):W=b.sibling;var N=f(g,b,y[I],x);if(N===null){b===null&&(b=W);break}t&&b&&N.alternate===null&&e(g,b),v=s(N,v,I),C===null?k=N:C.sibling=N,C=N,b=W}if(I===y.length)return n(g,b),ye&&Tr(g,I),k;if(b===null){for(;I<y.length;I++)b=h(g,y[I],x),b!==null&&(v=s(b,v,I),C===null?k=b:C.sibling=b,C=b);return ye&&Tr(g,I),k}for(b=r(g,b);I<y.length;I++)W=d(b,g,I,y[I],x),W!==null&&(t&&W.alternate!==null&&b.delete(W.key===null?I:W.key),v=s(W,v,I),C===null?k=W:C.sibling=W,C=W);return t&&b.forEach(function(q){return e(g,q)}),ye&&Tr(g,I),k}function m(g,v,y,x){var k=ss(y);if(typeof k!="function")throw Error(E(150));if(y=k.call(y),y==null)throw Error(E(151));for(var C=k=null,b=v,I=v=0,W=null,N=y.next();b!==null&&!N.done;I++,N=y.next()){b.index>I?(W=b,b=null):W=b.sibling;var q=f(g,b,N.value,x);if(q===null){b===null&&(b=W);break}t&&b&&q.alternate===null&&e(g,b),v=s(q,v,I),C===null?k=q:C.sibling=q,C=q,b=W}if(N.done)return n(g,b),ye&&Tr(g,I),k;if(b===null){for(;!N.done;I++,N=y.next())N=h(g,N.value,x),N!==null&&(v=s(N,v,I),C===null?k=N:C.sibling=N,C=N);return ye&&Tr(g,I),k}for(b=r(g,b);!N.done;I++,N=y.next())N=d(b,g,I,N.value,x),N!==null&&(t&&N.alternate!==null&&b.delete(N.key===null?I:N.key),v=s(N,v,I),C===null?k=N:C.sibling=N,C=N);return t&&b.forEach(function(j){return e(g,j)}),ye&&Tr(g,I),k}function w(g,v,y,x){if(typeof y=="object"&&y!==null&&y.type===li&&y.key===null&&(y=y.props.children),typeof y=="object"&&y!==null){switch(y.$$typeof){case Bo:e:{for(var k=y.key,C=v;C!==null;){if(C.key===k){if(k=y.type,k===li){if(C.tag===7){n(g,C.sibling),v=i(C,y.props.children),v.return=g,g=v;break e}}else if(C.elementType===k||typeof k=="object"&&k!==null&&k.$$typeof===Fn&&Op(k)===C.type){n(g,C.sibling),v=i(C,y.props),v.ref=us(g,C,y),v.return=g,g=v;break e}n(g,C);break}else e(g,C);C=C.sibling}y.type===li?(v=Br(y.props.children,g.mode,x,y.key),v.return=g,g=v):(x=Ll(y.type,y.key,y.props,null,g.mode,x),x.ref=us(g,v,y),x.return=g,g=x)}return o(g);case oi:e:{for(C=y.key;v!==null;){if(v.key===C)if(v.tag===4&&v.stateNode.containerInfo===y.containerInfo&&v.stateNode.implementation===y.implementation){n(g,v.sibling),v=i(v,y.children||[]),v.return=g,g=v;break e}else{n(g,v);break}else e(g,v);v=v.sibling}v=Vc(y,g.mode,x),v.return=g,g=v}return o(g);case Fn:return C=y._init,w(g,v,C(y._payload),x)}if(Ss(y))return p(g,v,y,x);if(ss(y))return m(g,v,y,x);Go(g,y)}return typeof y=="string"&&y!==""||typeof y=="number"?(y=""+y,v!==null&&v.tag===6?(n(g,v.sibling),v=i(v,y),v.return=g,g=v):(n(g,v),v=jc(y,g.mode,x),v.return=g,g=v),o(g)):n(g,v)}return w}var qi=av(!0),cv=av(!1),na=yr(null),ra=null,mi=null,Of=null;function Nf(){Of=mi=ra=null}function Bf(t){var e=na.current;ve(na),t._currentValue=e}function Xu(t,e,n){for(;t!==null;){var r=t.alternate;if((t.childLanes&e)!==e?(t.childLanes|=e,r!==null&&(r.childLanes|=e)):r!==null&&(r.childLanes&e)!==e&&(r.childLanes|=e),t===n)break;t=t.return}}function Ri(t,e){ra=t,Of=mi=null,t=t.dependencies,t!==null&&t.firstContext!==null&&(t.lanes&e&&(mt=!0),t.firstContext=null)}function Wt(t){var e=t._currentValue;if(Of!==t)if(t={context:t,memoizedValue:e,next:null},mi===null){if(ra===null)throw Error(E(308));mi=t,ra.dependencies={lanes:0,firstContext:t}}else mi=mi.next=t;return e}var Er=null;function _f(t){Er===null?Er=[t]:Er.push(t)}function uv(t,e,n,r){var i=e.interleaved;return i===null?(n.next=n,_f(e)):(n.next=i.next,i.next=n),e.interleaved=n,Dn(t,r)}function Dn(t,e){t.lanes|=e;var n=t.alternate;for(n!==null&&(n.lanes|=e),n=t,t=t.return;t!==null;)t.childLanes|=e,n=t.alternate,n!==null&&(n.childLanes|=e),n=t,t=t.return;return n.tag===3?n.stateNode:null}var zn=!1;function qf(t){t.updateQueue={baseState:t.memoizedState,firstBaseUpdate:null,lastBaseUpdate:null,shared:{pending:null,interleaved:null,lanes:0},effects:null}}function hv(t,e){t=t.updateQueue,e.updateQueue===t&&(e.updateQueue={baseState:t.baseState,firstBaseUpdate:t.firstBaseUpdate,lastBaseUpdate:t.lastBaseUpdate,shared:t.shared,effects:t.effects})}function An(t,e){return{eventTime:t,lane:e,tag:0,payload:null,callback:null,next:null}}function rr(t,e,n){var r=t.updateQueue;if(r===null)return null;if(r=r.shared,ne&2){var i=r.pending;return i===null?e.next=e:(e.next=i.next,i.next=e),r.pending=e,Dn(t,n)}return i=r.interleaved,i===null?(e.next=e,_f(r)):(e.next=i.next,i.next=e),r.interleaved=e,Dn(t,n)}function Rl(t,e,n){if(e=e.updateQueue,e!==null&&(e=e.shared,(n&4194240)!==0)){var r=e.lanes;r&=t.pendingLanes,n|=r,e.lanes=n,Cf(t,n)}}function Np(t,e){var n=t.updateQueue,r=t.alternate;if(r!==null&&(r=r.updateQueue,n===r)){var i=null,s=null;if(n=n.firstBaseUpdate,n!==null){do{var o={eventTime:n.eventTime,lane:n.lane,tag:n.tag,payload:n.payload,callback:n.callback,next:null};s===null?i=s=o:s=s.next=o,n=n.next}while(n!==null);s===null?i=s=e:s=s.next=e}else i=s=e;n={baseState:r.baseState,firstBaseUpdate:i,lastBaseUpdate:s,shared:r.shared,effects:r.effects},t.updateQueue=n;return}t=n.lastBaseUpdate,t===null?n.firstBaseUpdate=e:t.next=e,n.lastBaseUpdate=e}function ia(t,e,n,r){var i=t.updateQueue;zn=!1;var s=i.firstBaseUpdate,o=i.lastBaseUpdate,l=i.shared.pending;if(l!==null){i.shared.pending=null;var a=l,c=a.next;a.next=null,o===null?s=c:o.next=c,o=a;var u=t.alternate;u!==null&&(u=u.updateQueue,l=u.lastBaseUpdate,l!==o&&(l===null?u.firstBaseUpdate=c:l.next=c,u.lastBaseUpdate=a))}if(s!==null){var h=i.baseState;o=0,u=c=a=null,l=s;do{var f=l.lane,d=l.eventTime;if((r&f)===f){u!==null&&(u=u.next={eventTime:d,lane:0,tag:l.tag,payload:l.payload,callback:l.callback,next:null});e:{var p=t,m=l;switch(f=e,d=n,m.tag){case 1:if(p=m.payload,typeof p=="function"){h=p.call(d,h,f);break e}h=p;break e;case 3:p.flags=p.flags&-65537|128;case 0:if(p=m.payload,f=typeof p=="function"?p.call(d,h,f):p,f==null)break e;h=be({},h,f);break e;case 2:zn=!0}}l.callback!==null&&l.lane!==0&&(t.flags|=64,f=i.effects,f===null?i.effects=[l]:f.push(l))}else d={eventTime:d,lane:f,tag:l.tag,payload:l.payload,callback:l.callback,next:null},u===null?(c=u=d,a=h):u=u.next=d,o|=f;if(l=l.next,l===null){if(l=i.shared.pending,l===null)break;f=l,l=f.next,f.next=null,i.lastBaseUpdate=f,i.shared.pending=null}}while(!0);if(u===null&&(a=h),i.baseState=a,i.firstBaseUpdate=c,i.lastBaseUpdate=u,e=i.shared.interleaved,e!==null){i=e;do o|=i.lane,i=i.next;while(i!==e)}else s===null&&(i.shared.lanes=0);Vr|=o,t.lanes=o,t.memoizedState=h}}function Bp(t,e,n){if(t=e.effects,e.effects=null,t!==null)for(e=0;e<t.length;e++){var r=t[e],i=r.callback;if(i!==null){if(r.callback=null,r=n,typeof i!="function")throw Error(E(191,i));i.call(r)}}}var Ro={},Sn=yr(Ro),eo=yr(Ro),to=yr(Ro);function Lr(t){if(t===Ro)throw Error(E(174));return t}function Wf(t,e){switch(fe(to,e),fe(eo,t),fe(Sn,Ro),t=e.nodeType,t){case 9:case 11:e=(e=e.documentElement)?e.namespaceURI:Eu(null,"");break;default:t=t===8?e.parentNode:e,e=t.namespaceURI||null,t=t.tagName,e=Eu(e,t)}ve(Sn),fe(Sn,e)}function Wi(){ve(Sn),ve(eo),ve(to)}function fv(t){Lr(to.current);var e=Lr(Sn.current),n=Eu(e,t.type);e!==n&&(fe(eo,t),fe(Sn,n))}function Ff(t){eo.current===t&&(ve(Sn),ve(eo))}var xe=yr(0);function sa(t){for(var e=t;e!==null;){if(e.tag===13){var n=e.memoizedState;if(n!==null&&(n=n.dehydrated,n===null||n.data==="$?"||n.data==="$!"))return e}else if(e.tag===19&&e.memoizedProps.revealOrder!==void 0){if(e.flags&128)return e}else if(e.child!==null){e.child.return=e,e=e.child;continue}if(e===t)break;for(;e.sibling===null;){if(e.return===null||e.return===t)return null;e=e.return}e.sibling.return=e.return,e=e.sibling}return null}var Bc=[];function zf(){for(var t=0;t<Bc.length;t++)Bc[t]._workInProgressVersionPrimary=null;Bc.length=0}var Ml=Bn.ReactCurrentDispatcher,_c=Bn.ReactCurrentBatchConfig,jr=0,Ce=null,Le=null,Be=null,oa=!1,Es=!1,no=0,Mx=0;function Qe(){throw Error(E(321))}function jf(t,e){if(e===null)return!1;for(var n=0;n<e.length&&n<t.length;n++)if(!Xt(t[n],e[n]))return!1;return!0}function Vf(t,e,n,r,i,s){if(jr=s,Ce=e,e.memoizedState=null,e.updateQueue=null,e.lanes=0,Ml.current=t===null||t.memoizedState===null?Ex:Lx,t=n(r,i),Es){s=0;do{if(Es=!1,no=0,25<=s)throw Error(E(301));s+=1,Be=Le=null,e.updateQueue=null,Ml.current=Dx,t=n(r,i)}while(Es)}if(Ml.current=la,e=Le!==null&&Le.next!==null,jr=0,Be=Le=Ce=null,oa=!1,e)throw Error(E(300));return t}function Hf(){var t=no!==0;return no=0,t}function cn(){var t={memoizedState:null,baseState:null,baseQueue:null,queue:null,next:null};return Be===null?Ce.memoizedState=Be=t:Be=Be.next=t,Be}function Ft(){if(Le===null){var t=Ce.alternate;t=t!==null?t.memoizedState:null}else t=Le.next;var e=Be===null?Ce.memoizedState:Be.next;if(e!==null)Be=e,Le=t;else{if(t===null)throw Error(E(310));Le=t,t={memoizedState:Le.memoizedState,baseState:Le.baseState,baseQueue:Le.baseQueue,queue:Le.queue,next:null},Be===null?Ce.memoizedState=Be=t:Be=Be.next=t}return Be}function ro(t,e){return typeof e=="function"?e(t):e}function qc(t){var e=Ft(),n=e.queue;if(n===null)throw Error(E(311));n.lastRenderedReducer=t;var r=Le,i=r.baseQueue,s=n.pending;if(s!==null){if(i!==null){var o=i.next;i.next=s.next,s.next=o}r.baseQueue=i=s,n.pending=null}if(i!==null){s=i.next,r=r.baseState;var l=o=null,a=null,c=s;do{var u=c.lane;if((jr&u)===u)a!==null&&(a=a.next={lane:0,action:c.action,hasEagerState:c.hasEagerState,eagerState:c.eagerState,next:null}),r=c.hasEagerState?c.eagerState:t(r,c.action);else{var h={lane:u,action:c.action,hasEagerState:c.hasEagerState,eagerState:c.eagerState,next:null};a===null?(l=a=h,o=r):a=a.next=h,Ce.lanes|=u,Vr|=u}c=c.next}while(c!==null&&c!==s);a===null?o=r:a.next=l,Xt(r,e.memoizedState)||(mt=!0),e.memoizedState=r,e.baseState=o,e.baseQueue=a,n.lastRenderedState=r}if(t=n.interleaved,t!==null){i=t;do s=i.lane,Ce.lanes|=s,Vr|=s,i=i.next;while(i!==t)}else i===null&&(n.lanes=0);return[e.memoizedState,n.dispatch]}function Wc(t){var e=Ft(),n=e.queue;if(n===null)throw Error(E(311));n.lastRenderedReducer=t;var r=n.dispatch,i=n.pending,s=e.memoizedState;if(i!==null){n.pending=null;var o=i=i.next;do s=t(s,o.action),o=o.next;while(o!==i);Xt(s,e.memoizedState)||(mt=!0),e.memoizedState=s,e.baseQueue===null&&(e.baseState=s),n.lastRenderedState=s}return[s,r]}function dv(){}function pv(t,e){var n=Ce,r=Ft(),i=e(),s=!Xt(r.memoizedState,i);if(s&&(r.memoizedState=i,mt=!0),r=r.queue,Uf(vv.bind(null,n,r,t),[t]),r.getSnapshot!==e||s||Be!==null&&Be.memoizedState.tag&1){if(n.flags|=2048,io(9,gv.bind(null,n,r,i,e),void 0,null),We===null)throw Error(E(349));jr&30||mv(n,e,i)}return i}function mv(t,e,n){t.flags|=16384,t={getSnapshot:e,value:n},e=Ce.updateQueue,e===null?(e={lastEffect:null,stores:null},Ce.updateQueue=e,e.stores=[t]):(n=e.stores,n===null?e.stores=[t]:n.push(t))}function gv(t,e,n,r){e.value=n,e.getSnapshot=r,yv(e)&&wv(t)}function vv(t,e,n){return n(function(){yv(e)&&wv(t)})}function yv(t){var e=t.getSnapshot;t=t.value;try{var n=e();return!Xt(t,n)}catch{return!0}}function wv(t){var e=Dn(t,1);e!==null&&Yt(e,t,1,-1)}function _p(t){var e=cn();return typeof t=="function"&&(t=t()),e.memoizedState=e.baseState=t,t={pending:null,interleaved:null,lanes:0,dispatch:null,lastRenderedReducer:ro,lastRenderedState:t},e.queue=t,t=t.dispatch=Px.bind(null,Ce,t),[e.memoizedState,t]}function io(t,e,n,r){return t={tag:t,create:e,destroy:n,deps:r,next:null},e=Ce.updateQueue,e===null?(e={lastEffect:null,stores:null},Ce.updateQueue=e,e.lastEffect=t.next=t):(n=e.lastEffect,n===null?e.lastEffect=t.next=t:(r=n.next,n.next=t,t.next=r,e.lastEffect=t)),t}function Sv(){return Ft().memoizedState}function Il(t,e,n,r){var i=cn();Ce.flags|=t,i.memoizedState=io(1|e,n,void 0,r===void 0?null:r)}function ja(t,e,n,r){var i=Ft();r=r===void 0?null:r;var s=void 0;if(Le!==null){var o=Le.memoizedState;if(s=o.destroy,r!==null&&jf(r,o.deps)){i.memoizedState=io(e,n,s,r);return}}Ce.flags|=t,i.memoizedState=io(1|e,n,s,r)}function qp(t,e){return Il(8390656,8,t,e)}function Uf(t,e){return ja(2048,8,t,e)}function kv(t,e){return ja(4,2,t,e)}function xv(t,e){return ja(4,4,t,e)}function Cv(t,e){if(typeof e=="function")return t=t(),e(t),function(){e(null)};if(e!=null)return t=t(),e.current=t,function(){e.current=null}}function bv(t,e,n){return n=n!=null?n.concat([t]):null,ja(4,4,Cv.bind(null,e,t),n)}function $f(){}function Tv(t,e){var n=Ft();e=e===void 0?null:e;var r=n.memoizedState;return r!==null&&e!==null&&jf(e,r[1])?r[0]:(n.memoizedState=[t,e],t)}function Rv(t,e){var n=Ft();e=e===void 0?null:e;var r=n.memoizedState;return r!==null&&e!==null&&jf(e,r[1])?r[0]:(t=t(),n.memoizedState=[t,e],t)}function Mv(t,e,n){return jr&21?(Xt(n,e)||(n=L0(),Ce.lanes|=n,Vr|=n,t.baseState=!0),e):(t.baseState&&(t.baseState=!1,mt=!0),t.memoizedState=n)}function Ix(t,e){var n=oe;oe=n!==0&&4>n?n:4,t(!0);var r=_c.transition;_c.transition={};try{t(!1),e()}finally{oe=n,_c.transition=r}}function Iv(){return Ft().memoizedState}function Ax(t,e,n){var r=sr(t);if(n={lane:r,action:n,hasEagerState:!1,eagerState:null,next:null},Av(t))Pv(e,n);else if(n=uv(t,e,n,r),n!==null){var i=ot();Yt(n,t,r,i),Ev(n,e,r)}}function Px(t,e,n){var r=sr(t),i={lane:r,action:n,hasEagerState:!1,eagerState:null,next:null};if(Av(t))Pv(e,i);else{var s=t.alternate;if(t.lanes===0&&(s===null||s.lanes===0)&&(s=e.lastRenderedReducer,s!==null))try{var o=e.lastRenderedState,l=s(o,n);if(i.hasEagerState=!0,i.eagerState=l,Xt(l,o)){var a=e.interleaved;a===null?(i.next=i,_f(e)):(i.next=a.next,a.next=i),e.interleaved=i;return}}catch{}finally{}n=uv(t,e,i,r),n!==null&&(i=ot(),Yt(n,t,r,i),Ev(n,e,r))}}function Av(t){var e=t.alternate;return t===Ce||e!==null&&e===Ce}function Pv(t,e){Es=oa=!0;var n=t.pending;n===null?e.next=e:(e.next=n.next,n.next=e),t.pending=e}function Ev(t,e,n){if(n&4194240){var r=e.lanes;r&=t.pendingLanes,n|=r,e.lanes=n,Cf(t,n)}}var la={readContext:Wt,useCallback:Qe,useContext:Qe,useEffect:Qe,useImperativeHandle:Qe,useInsertionEffect:Qe,useLayoutEffect:Qe,useMemo:Qe,useReducer:Qe,useRef:Qe,useState:Qe,useDebugValue:Qe,useDeferredValue:Qe,useTransition:Qe,useMutableSource:Qe,useSyncExternalStore:Qe,useId:Qe,unstable_isNewReconciler:!1},Ex={readContext:Wt,useCallback:function(t,e){return cn().memoizedState=[t,e===void 0?null:e],t},useContext:Wt,useEffect:qp,useImperativeHandle:function(t,e,n){return n=n!=null?n.concat([t]):null,Il(4194308,4,Cv.bind(null,e,t),n)},useLayoutEffect:function(t,e){return Il(4194308,4,t,e)},useInsertionEffect:function(t,e){return Il(4,2,t,e)},useMemo:function(t,e){var n=cn();return e=e===void 0?null:e,t=t(),n.memoizedState=[t,e],t},useReducer:function(t,e,n){var r=cn();return e=n!==void 0?n(e):e,r.memoizedState=r.baseState=e,t={pending:null,interleaved:null,lanes:0,dispatch:null,lastRenderedReducer:t,lastRenderedState:e},r.queue=t,t=t.dispatch=Ax.bind(null,Ce,t),[r.memoizedState,t]},useRef:function(t){var e=cn();return t={current:t},e.memoizedState=t},useState:_p,useDebugValue:$f,useDeferredValue:function(t){return cn().memoizedState=t},useTransition:function(){var t=_p(!1),e=t[0];return t=Ix.bind(null,t[1]),cn().memoizedState=t,[e,t]},useMutableSource:function(){},useSyncExternalStore:function(t,e,n){var r=Ce,i=cn();if(ye){if(n===void 0)throw Error(E(407));n=n()}else{if(n=e(),We===null)throw Error(E(349));jr&30||mv(r,e,n)}i.memoizedState=n;var s={value:n,getSnapshot:e};return i.queue=s,qp(vv.bind(null,r,s,t),[t]),r.flags|=2048,io(9,gv.bind(null,r,s,n,e),void 0,null),n},useId:function(){var t=cn(),e=We.identifierPrefix;if(ye){var n=In,r=Mn;n=(r&~(1<<32-Qt(r)-1)).toString(32)+n,e=":"+e+"R"+n,n=no++,0<n&&(e+="H"+n.toString(32)),e+=":"}else n=Mx++,e=":"+e+"r"+n.toString(32)+":";return t.memoizedState=e},unstable_isNewReconciler:!1},Lx={readContext:Wt,useCallback:Tv,useContext:Wt,useEffect:Uf,useImperativeHandle:bv,useInsertionEffect:kv,useLayoutEffect:xv,useMemo:Rv,useReducer:qc,useRef:Sv,useState:function(){return qc(ro)},useDebugValue:$f,useDeferredValue:function(t){var e=Ft();return Mv(e,Le.memoizedState,t)},useTransition:function(){var t=qc(ro)[0],e=Ft().memoizedState;return[t,e]},useMutableSource:dv,useSyncExternalStore:pv,useId:Iv,unstable_isNewReconciler:!1},Dx={readContext:Wt,useCallback:Tv,useContext:Wt,useEffect:Uf,useImperativeHandle:bv,useInsertionEffect:kv,useLayoutEffect:xv,useMemo:Rv,useReducer:Wc,useRef:Sv,useState:function(){return Wc(ro)},useDebugValue:$f,useDeferredValue:function(t){var e=Ft();return Le===null?e.memoizedState=t:Mv(e,Le.memoizedState,t)},useTransition:function(){var t=Wc(ro)[0],e=Ft().memoizedState;return[t,e]},useMutableSource:dv,useSyncExternalStore:pv,useId:Iv,unstable_isNewReconciler:!1};function Vt(t,e){if(t&&t.defaultProps){e=be({},e),t=t.defaultProps;for(var n in t)e[n]===void 0&&(e[n]=t[n]);return e}return e}function Ju(t,e,n,r){e=t.memoizedState,n=n(r,e),n=n==null?e:be({},e,n),t.memoizedState=n,t.lanes===0&&(t.updateQueue.baseState=n)}var Va={isMounted:function(t){return(t=t._reactInternals)?Jr(t)===t:!1},enqueueSetState:function(t,e,n){t=t._reactInternals;var r=ot(),i=sr(t),s=An(r,i);s.payload=e,n!=null&&(s.callback=n),e=rr(t,s,i),e!==null&&(Yt(e,t,i,r),Rl(e,t,i))},enqueueReplaceState:function(t,e,n){t=t._reactInternals;var r=ot(),i=sr(t),s=An(r,i);s.tag=1,s.payload=e,n!=null&&(s.callback=n),e=rr(t,s,i),e!==null&&(Yt(e,t,i,r),Rl(e,t,i))},enqueueForceUpdate:function(t,e){t=t._reactInternals;var n=ot(),r=sr(t),i=An(n,r);i.tag=2,e!=null&&(i.callback=e),e=rr(t,i,r),e!==null&&(Yt(e,t,r,n),Rl(e,t,r))}};function Wp(t,e,n,r,i,s,o){return t=t.stateNode,typeof t.shouldComponentUpdate=="function"?t.shouldComponentUpdate(r,s,o):e.prototype&&e.prototype.isPureReactComponent?!Ys(n,r)||!Ys(i,s):!0}function Lv(t,e,n){var r=!1,i=cr,s=e.contextType;return typeof s=="object"&&s!==null?s=Wt(s):(i=yt(e)?Fr:Ze.current,r=e.contextTypes,s=(r=r!=null)?Bi(t,i):cr),e=new e(n,s),t.memoizedState=e.state!==null&&e.state!==void 0?e.state:null,e.updater=Va,t.stateNode=e,e._reactInternals=t,r&&(t=t.stateNode,t.__reactInternalMemoizedUnmaskedChildContext=i,t.__reactInternalMemoizedMaskedChildContext=s),e}function Fp(t,e,n,r){t=e.state,typeof e.componentWillReceiveProps=="function"&&e.componentWillReceiveProps(n,r),typeof e.UNSAFE_componentWillReceiveProps=="function"&&e.UNSAFE_componentWillReceiveProps(n,r),e.state!==t&&Va.enqueueReplaceState(e,e.state,null)}function Zu(t,e,n,r){var i=t.stateNode;i.props=n,i.state=t.memoizedState,i.refs={},qf(t);var s=e.contextType;typeof s=="object"&&s!==null?i.context=Wt(s):(s=yt(e)?Fr:Ze.current,i.context=Bi(t,s)),i.state=t.memoizedState,s=e.getDerivedStateFromProps,typeof s=="function"&&(Ju(t,e,s,n),i.state=t.memoizedState),typeof e.getDerivedStateFromProps=="function"||typeof i.getSnapshotBeforeUpdate=="function"||typeof i.UNSAFE_componentWillMount!="function"&&typeof i.componentWillMount!="function"||(e=i.state,typeof i.componentWillMount=="function"&&i.componentWillMount(),typeof i.UNSAFE_componentWillMount=="function"&&i.UNSAFE_componentWillMount(),e!==i.state&&Va.enqueueReplaceState(i,i.state,null),ia(t,n,i,r),i.state=t.memoizedState),typeof i.componentDidMount=="function"&&(t.flags|=4194308)}function Fi(t,e){try{var n="",r=e;do n+=lk(r),r=r.return;while(r);var i=n}catch(s){i=`
Error generating stack: `+s.message+`
`+s.stack}return{value:t,source:e,stack:i,digest:null}}function Fc(t,e,n){return{value:t,source:null,stack:n??null,digest:e??null}}function eh(t,e){try{console.error(e.value)}catch(n){setTimeout(function(){throw n})}}var Ox=typeof WeakMap=="function"?WeakMap:Map;function Dv(t,e,n){n=An(-1,n),n.tag=3,n.payload={element:null};var r=e.value;return n.callback=function(){ca||(ca=!0,uh=r),eh(t,e)},n}function Ov(t,e,n){n=An(-1,n),n.tag=3;var r=t.type.getDerivedStateFromError;if(typeof r=="function"){var i=e.value;n.payload=function(){return r(i)},n.callback=function(){eh(t,e)}}var s=t.stateNode;return s!==null&&typeof s.componentDidCatch=="function"&&(n.callback=function(){eh(t,e),typeof r!="function"&&(ir===null?ir=new Set([this]):ir.add(this));var o=e.stack;this.componentDidCatch(e.value,{componentStack:o!==null?o:""})}),n}function zp(t,e,n){var r=t.pingCache;if(r===null){r=t.pingCache=new Ox;var i=new Set;r.set(e,i)}else i=r.get(e),i===void 0&&(i=new Set,r.set(e,i));i.has(n)||(i.add(n),t=Kx.bind(null,t,e,n),e.then(t,t))}function jp(t){do{var e;if((e=t.tag===13)&&(e=t.memoizedState,e=e!==null?e.dehydrated!==null:!0),e)return t;t=t.return}while(t!==null);return null}function Vp(t,e,n,r,i){return t.mode&1?(t.flags|=65536,t.lanes=i,t):(t===e?t.flags|=65536:(t.flags|=128,n.flags|=131072,n.flags&=-52805,n.tag===1&&(n.alternate===null?n.tag=17:(e=An(-1,1),e.tag=2,rr(n,e,1))),n.lanes|=1),t)}var Nx=Bn.ReactCurrentOwner,mt=!1;function nt(t,e,n,r){e.child=t===null?cv(e,null,n,r):qi(e,t.child,n,r)}function Hp(t,e,n,r,i){n=n.render;var s=e.ref;return Ri(e,i),r=Vf(t,e,n,r,s,i),n=Hf(),t!==null&&!mt?(e.updateQueue=t.updateQueue,e.flags&=-2053,t.lanes&=~i,On(t,e,i)):(ye&&n&&Ef(e),e.flags|=1,nt(t,e,r,i),e.child)}function Up(t,e,n,r,i){if(t===null){var s=n.type;return typeof s=="function"&&!ed(s)&&s.defaultProps===void 0&&n.compare===null&&n.defaultProps===void 0?(e.tag=15,e.type=s,Nv(t,e,s,r,i)):(t=Ll(n.type,null,r,e,e.mode,i),t.ref=e.ref,t.return=e,e.child=t)}if(s=t.child,!(t.lanes&i)){var o=s.memoizedProps;if(n=n.compare,n=n!==null?n:Ys,n(o,r)&&t.ref===e.ref)return On(t,e,i)}return e.flags|=1,t=or(s,r),t.ref=e.ref,t.return=e,e.child=t}function Nv(t,e,n,r,i){if(t!==null){var s=t.memoizedProps;if(Ys(s,r)&&t.ref===e.ref)if(mt=!1,e.pendingProps=r=s,(t.lanes&i)!==0)t.flags&131072&&(mt=!0);else return e.lanes=t.lanes,On(t,e,i)}return th(t,e,n,r,i)}function Bv(t,e,n){var r=e.pendingProps,i=r.children,s=t!==null?t.memoizedState:null;if(r.mode==="hidden")if(!(e.mode&1))e.memoizedState={baseLanes:0,cachePool:null,transitions:null},fe(vi,xt),xt|=n;else{if(!(n&1073741824))return t=s!==null?s.baseLanes|n:n,e.lanes=e.childLanes=1073741824,e.memoizedState={baseLanes:t,cachePool:null,transitions:null},e.updateQueue=null,fe(vi,xt),xt|=t,null;e.memoizedState={baseLanes:0,cachePool:null,transitions:null},r=s!==null?s.baseLanes:n,fe(vi,xt),xt|=r}else s!==null?(r=s.baseLanes|n,e.memoizedState=null):r=n,fe(vi,xt),xt|=r;return nt(t,e,i,n),e.child}function _v(t,e){var n=e.ref;(t===null&&n!==null||t!==null&&t.ref!==n)&&(e.flags|=512,e.flags|=2097152)}function th(t,e,n,r,i){var s=yt(n)?Fr:Ze.current;return s=Bi(e,s),Ri(e,i),n=Vf(t,e,n,r,s,i),r=Hf(),t!==null&&!mt?(e.updateQueue=t.updateQueue,e.flags&=-2053,t.lanes&=~i,On(t,e,i)):(ye&&r&&Ef(e),e.flags|=1,nt(t,e,n,i),e.child)}function $p(t,e,n,r,i){if(yt(n)){var s=!0;Zl(e)}else s=!1;if(Ri(e,i),e.stateNode===null)Al(t,e),Lv(e,n,r),Zu(e,n,r,i),r=!0;else if(t===null){var o=e.stateNode,l=e.memoizedProps;o.props=l;var a=o.context,c=n.contextType;typeof c=="object"&&c!==null?c=Wt(c):(c=yt(n)?Fr:Ze.current,c=Bi(e,c));var u=n.getDerivedStateFromProps,h=typeof u=="function"||typeof o.getSnapshotBeforeUpdate=="function";h||typeof o.UNSAFE_componentWillReceiveProps!="function"&&typeof o.componentWillReceiveProps!="function"||(l!==r||a!==c)&&Fp(e,o,r,c),zn=!1;var f=e.memoizedState;o.state=f,ia(e,r,o,i),a=e.memoizedState,l!==r||f!==a||vt.current||zn?(typeof u=="function"&&(Ju(e,n,u,r),a=e.memoizedState),(l=zn||Wp(e,n,l,r,f,a,c))?(h||typeof o.UNSAFE_componentWillMount!="function"&&typeof o.componentWillMount!="function"||(typeof o.componentWillMount=="function"&&o.componentWillMount(),typeof o.UNSAFE_componentWillMount=="function"&&o.UNSAFE_componentWillMount()),typeof o.componentDidMount=="function"&&(e.flags|=4194308)):(typeof o.componentDidMount=="function"&&(e.flags|=4194308),e.memoizedProps=r,e.memoizedState=a),o.props=r,o.state=a,o.context=c,r=l):(typeof o.componentDidMount=="function"&&(e.flags|=4194308),r=!1)}else{o=e.stateNode,hv(t,e),l=e.memoizedProps,c=e.type===e.elementType?l:Vt(e.type,l),o.props=c,h=e.pendingProps,f=o.context,a=n.contextType,typeof a=="object"&&a!==null?a=Wt(a):(a=yt(n)?Fr:Ze.current,a=Bi(e,a));var d=n.getDerivedStateFromProps;(u=typeof d=="function"||typeof o.getSnapshotBeforeUpdate=="function")||typeof o.UNSAFE_componentWillReceiveProps!="function"&&typeof o.componentWillReceiveProps!="function"||(l!==h||f!==a)&&Fp(e,o,r,a),zn=!1,f=e.memoizedState,o.state=f,ia(e,r,o,i);var p=e.memoizedState;l!==h||f!==p||vt.current||zn?(typeof d=="function"&&(Ju(e,n,d,r),p=e.memoizedState),(c=zn||Wp(e,n,c,r,f,p,a)||!1)?(u||typeof o.UNSAFE_componentWillUpdate!="function"&&typeof o.componentWillUpdate!="function"||(typeof o.componentWillUpdate=="function"&&o.componentWillUpdate(r,p,a),typeof o.UNSAFE_componentWillUpdate=="function"&&o.UNSAFE_componentWillUpdate(r,p,a)),typeof o.componentDidUpdate=="function"&&(e.flags|=4),typeof o.getSnapshotBeforeUpdate=="function"&&(e.flags|=1024)):(typeof o.componentDidUpdate!="function"||l===t.memoizedProps&&f===t.memoizedState||(e.flags|=4),typeof o.getSnapshotBeforeUpdate!="function"||l===t.memoizedProps&&f===t.memoizedState||(e.flags|=1024),e.memoizedProps=r,e.memoizedState=p),o.props=r,o.state=p,o.context=a,r=c):(typeof o.componentDidUpdate!="function"||l===t.memoizedProps&&f===t.memoizedState||(e.flags|=4),typeof o.getSnapshotBeforeUpdate!="function"||l===t.memoizedProps&&f===t.memoizedState||(e.flags|=1024),r=!1)}return nh(t,e,n,r,s,i)}function nh(t,e,n,r,i,s){_v(t,e);var o=(e.flags&128)!==0;if(!r&&!o)return i&&Ep(e,n,!1),On(t,e,s);r=e.stateNode,Nx.current=e;var l=o&&typeof n.getDerivedStateFromError!="function"?null:r.render();return e.flags|=1,t!==null&&o?(e.child=qi(e,t.child,null,s),e.child=qi(e,null,l,s)):nt(t,e,l,s),e.memoizedState=r.state,i&&Ep(e,n,!0),e.child}function qv(t){var e=t.stateNode;e.pendingContext?Pp(t,e.pendingContext,e.pendingContext!==e.context):e.context&&Pp(t,e.context,!1),Wf(t,e.containerInfo)}function Gp(t,e,n,r,i){return _i(),Df(i),e.flags|=256,nt(t,e,n,r),e.child}var rh={dehydrated:null,treeContext:null,retryLane:0};function ih(t){return{baseLanes:t,cachePool:null,transitions:null}}function Wv(t,e,n){var r=e.pendingProps,i=xe.current,s=!1,o=(e.flags&128)!==0,l;if((l=o)||(l=t!==null&&t.memoizedState===null?!1:(i&2)!==0),l?(s=!0,e.flags&=-129):(t===null||t.memoizedState!==null)&&(i|=1),fe(xe,i&1),t===null)return Yu(e),t=e.memoizedState,t!==null&&(t=t.dehydrated,t!==null)?(e.mode&1?t.data==="$!"?e.lanes=8:e.lanes=1073741824:e.lanes=1,null):(o=r.children,t=r.fallback,s?(r=e.mode,s=e.child,o={mode:"hidden",children:o},!(r&1)&&s!==null?(s.childLanes=0,s.pendingProps=o):s=$a(o,r,0,null),t=Br(t,r,n,null),s.return=e,t.return=e,s.sibling=t,e.child=s,e.child.memoizedState=ih(n),e.memoizedState=rh,t):Gf(e,o));if(i=t.memoizedState,i!==null&&(l=i.dehydrated,l!==null))return Bx(t,e,o,r,l,i,n);if(s){s=r.fallback,o=e.mode,i=t.child,l=i.sibling;var a={mode:"hidden",children:r.children};return!(o&1)&&e.child!==i?(r=e.child,r.childLanes=0,r.pendingProps=a,e.deletions=null):(r=or(i,a),r.subtreeFlags=i.subtreeFlags&14680064),l!==null?s=or(l,s):(s=Br(s,o,n,null),s.flags|=2),s.return=e,r.return=e,r.sibling=s,e.child=r,r=s,s=e.child,o=t.child.memoizedState,o=o===null?ih(n):{baseLanes:o.baseLanes|n,cachePool:null,transitions:o.transitions},s.memoizedState=o,s.childLanes=t.childLanes&~n,e.memoizedState=rh,r}return s=t.child,t=s.sibling,r=or(s,{mode:"visible",children:r.children}),!(e.mode&1)&&(r.lanes=n),r.return=e,r.sibling=null,t!==null&&(n=e.deletions,n===null?(e.deletions=[t],e.flags|=16):n.push(t)),e.child=r,e.memoizedState=null,r}function Gf(t,e){return e=$a({mode:"visible",children:e},t.mode,0,null),e.return=t,t.child=e}function Ko(t,e,n,r){return r!==null&&Df(r),qi(e,t.child,null,n),t=Gf(e,e.pendingProps.children),t.flags|=2,e.memoizedState=null,t}function Bx(t,e,n,r,i,s,o){if(n)return e.flags&256?(e.flags&=-257,r=Fc(Error(E(422))),Ko(t,e,o,r)):e.memoizedState!==null?(e.child=t.child,e.flags|=128,null):(s=r.fallback,i=e.mode,r=$a({mode:"visible",children:r.children},i,0,null),s=Br(s,i,o,null),s.flags|=2,r.return=e,s.return=e,r.sibling=s,e.child=r,e.mode&1&&qi(e,t.child,null,o),e.child.memoizedState=ih(o),e.memoizedState=rh,s);if(!(e.mode&1))return Ko(t,e,o,null);if(i.data==="$!"){if(r=i.nextSibling&&i.nextSibling.dataset,r)var l=r.dgst;return r=l,s=Error(E(419)),r=Fc(s,r,void 0),Ko(t,e,o,r)}if(l=(o&t.childLanes)!==0,mt||l){if(r=We,r!==null){switch(o&-o){case 4:i=2;break;case 16:i=8;break;case 64:case 128:case 256:case 512:case 1024:case 2048:case 4096:case 8192:case 16384:case 32768:case 65536:case 131072:case 262144:case 524288:case 1048576:case 2097152:case 4194304:case 8388608:case 16777216:case 33554432:case 67108864:i=32;break;case 536870912:i=268435456;break;default:i=0}i=i&(r.suspendedLanes|o)?0:i,i!==0&&i!==s.retryLane&&(s.retryLane=i,Dn(t,i),Yt(r,t,i,-1))}return Zf(),r=Fc(Error(E(421))),Ko(t,e,o,r)}return i.data==="$?"?(e.flags|=128,e.child=t.child,e=Qx.bind(null,t),i._reactRetry=e,null):(t=s.treeContext,bt=nr(i.nextSibling),Rt=e,ye=!0,Ut=null,t!==null&&(Pt[Et++]=Mn,Pt[Et++]=In,Pt[Et++]=zr,Mn=t.id,In=t.overflow,zr=e),e=Gf(e,r.children),e.flags|=4096,e)}function Kp(t,e,n){t.lanes|=e;var r=t.alternate;r!==null&&(r.lanes|=e),Xu(t.return,e,n)}function zc(t,e,n,r,i){var s=t.memoizedState;s===null?t.memoizedState={isBackwards:e,rendering:null,renderingStartTime:0,last:r,tail:n,tailMode:i}:(s.isBackwards=e,s.rendering=null,s.renderingStartTime=0,s.last=r,s.tail=n,s.tailMode=i)}function Fv(t,e,n){var r=e.pendingProps,i=r.revealOrder,s=r.tail;if(nt(t,e,r.children,n),r=xe.current,r&2)r=r&1|2,e.flags|=128;else{if(t!==null&&t.flags&128)e:for(t=e.child;t!==null;){if(t.tag===13)t.memoizedState!==null&&Kp(t,n,e);else if(t.tag===19)Kp(t,n,e);else if(t.child!==null){t.child.return=t,t=t.child;continue}if(t===e)break e;for(;t.sibling===null;){if(t.return===null||t.return===e)break e;t=t.return}t.sibling.return=t.return,t=t.sibling}r&=1}if(fe(xe,r),!(e.mode&1))e.memoizedState=null;else switch(i){case"forwards":for(n=e.child,i=null;n!==null;)t=n.alternate,t!==null&&sa(t)===null&&(i=n),n=n.sibling;n=i,n===null?(i=e.child,e.child=null):(i=n.sibling,n.sibling=null),zc(e,!1,i,n,s);break;case"backwards":for(n=null,i=e.child,e.child=null;i!==null;){if(t=i.alternate,t!==null&&sa(t)===null){e.child=i;break}t=i.sibling,i.sibling=n,n=i,i=t}zc(e,!0,n,null,s);break;case"together":zc(e,!1,null,null,void 0);break;default:e.memoizedState=null}return e.child}function Al(t,e){!(e.mode&1)&&t!==null&&(t.alternate=null,e.alternate=null,e.flags|=2)}function On(t,e,n){if(t!==null&&(e.dependencies=t.dependencies),Vr|=e.lanes,!(n&e.childLanes))return null;if(t!==null&&e.child!==t.child)throw Error(E(153));if(e.child!==null){for(t=e.child,n=or(t,t.pendingProps),e.child=n,n.return=e;t.sibling!==null;)t=t.sibling,n=n.sibling=or(t,t.pendingProps),n.return=e;n.sibling=null}return e.child}function _x(t,e,n){switch(e.tag){case 3:qv(e),_i();break;case 5:fv(e);break;case 1:yt(e.type)&&Zl(e);break;case 4:Wf(e,e.stateNode.containerInfo);break;case 10:var r=e.type._context,i=e.memoizedProps.value;fe(na,r._currentValue),r._currentValue=i;break;case 13:if(r=e.memoizedState,r!==null)return r.dehydrated!==null?(fe(xe,xe.current&1),e.flags|=128,null):n&e.child.childLanes?Wv(t,e,n):(fe(xe,xe.current&1),t=On(t,e,n),t!==null?t.sibling:null);fe(xe,xe.current&1);break;case 19:if(r=(n&e.childLanes)!==0,t.flags&128){if(r)return Fv(t,e,n);e.flags|=128}if(i=e.memoizedState,i!==null&&(i.rendering=null,i.tail=null,i.lastEffect=null),fe(xe,xe.current),r)break;return null;case 22:case 23:return e.lanes=0,Bv(t,e,n)}return On(t,e,n)}var zv,sh,jv,Vv;zv=function(t,e){for(var n=e.child;n!==null;){if(n.tag===5||n.tag===6)t.appendChild(n.stateNode);else if(n.tag!==4&&n.child!==null){n.child.return=n,n=n.child;continue}if(n===e)break;for(;n.sibling===null;){if(n.return===null||n.return===e)return;n=n.return}n.sibling.return=n.return,n=n.sibling}};sh=function(){};jv=function(t,e,n,r){var i=t.memoizedProps;if(i!==r){t=e.stateNode,Lr(Sn.current);var s=null;switch(n){case"input":i=Mu(t,i),r=Mu(t,r),s=[];break;case"select":i=be({},i,{value:void 0}),r=be({},r,{value:void 0}),s=[];break;case"textarea":i=Pu(t,i),r=Pu(t,r),s=[];break;default:typeof i.onClick!="function"&&typeof r.onClick=="function"&&(t.onclick=Xl)}Lu(n,r);var o;n=null;for(c in i)if(!r.hasOwnProperty(c)&&i.hasOwnProperty(c)&&i[c]!=null)if(c==="style"){var l=i[c];for(o in l)l.hasOwnProperty(o)&&(n||(n={}),n[o]="")}else c!=="dangerouslySetInnerHTML"&&c!=="children"&&c!=="suppressContentEditableWarning"&&c!=="suppressHydrationWarning"&&c!=="autoFocus"&&(Vs.hasOwnProperty(c)?s||(s=[]):(s=s||[]).push(c,null));for(c in r){var a=r[c];if(l=i!=null?i[c]:void 0,r.hasOwnProperty(c)&&a!==l&&(a!=null||l!=null))if(c==="style")if(l){for(o in l)!l.hasOwnProperty(o)||a&&a.hasOwnProperty(o)||(n||(n={}),n[o]="");for(o in a)a.hasOwnProperty(o)&&l[o]!==a[o]&&(n||(n={}),n[o]=a[o])}else n||(s||(s=[]),s.push(c,n)),n=a;else c==="dangerouslySetInnerHTML"?(a=a?a.__html:void 0,l=l?l.__html:void 0,a!=null&&l!==a&&(s=s||[]).push(c,a)):c==="children"?typeof a!="string"&&typeof a!="number"||(s=s||[]).push(c,""+a):c!=="suppressContentEditableWarning"&&c!=="suppressHydrationWarning"&&(Vs.hasOwnProperty(c)?(a!=null&&c==="onScroll"&&ge("scroll",t),s||l===a||(s=[])):(s=s||[]).push(c,a))}n&&(s=s||[]).push("style",n);var c=s;(e.updateQueue=c)&&(e.flags|=4)}};Vv=function(t,e,n,r){n!==r&&(e.flags|=4)};function hs(t,e){if(!ye)switch(t.tailMode){case"hidden":e=t.tail;for(var n=null;e!==null;)e.alternate!==null&&(n=e),e=e.sibling;n===null?t.tail=null:n.sibling=null;break;case"collapsed":n=t.tail;for(var r=null;n!==null;)n.alternate!==null&&(r=n),n=n.sibling;r===null?e||t.tail===null?t.tail=null:t.tail.sibling=null:r.sibling=null}}function Ye(t){var e=t.alternate!==null&&t.alternate.child===t.child,n=0,r=0;if(e)for(var i=t.child;i!==null;)n|=i.lanes|i.childLanes,r|=i.subtreeFlags&14680064,r|=i.flags&14680064,i.return=t,i=i.sibling;else for(i=t.child;i!==null;)n|=i.lanes|i.childLanes,r|=i.subtreeFlags,r|=i.flags,i.return=t,i=i.sibling;return t.subtreeFlags|=r,t.childLanes=n,e}function qx(t,e,n){var r=e.pendingProps;switch(Lf(e),e.tag){case 2:case 16:case 15:case 0:case 11:case 7:case 8:case 12:case 9:case 14:return Ye(e),null;case 1:return yt(e.type)&&Jl(),Ye(e),null;case 3:return r=e.stateNode,Wi(),ve(vt),ve(Ze),zf(),r.pendingContext&&(r.context=r.pendingContext,r.pendingContext=null),(t===null||t.child===null)&&($o(e)?e.flags|=4:t===null||t.memoizedState.isDehydrated&&!(e.flags&256)||(e.flags|=1024,Ut!==null&&(dh(Ut),Ut=null))),sh(t,e),Ye(e),null;case 5:Ff(e);var i=Lr(to.current);if(n=e.type,t!==null&&e.stateNode!=null)jv(t,e,n,r,i),t.ref!==e.ref&&(e.flags|=512,e.flags|=2097152);else{if(!r){if(e.stateNode===null)throw Error(E(166));return Ye(e),null}if(t=Lr(Sn.current),$o(e)){r=e.stateNode,n=e.type;var s=e.memoizedProps;switch(r[un]=e,r[Zs]=s,t=(e.mode&1)!==0,n){case"dialog":ge("cancel",r),ge("close",r);break;case"iframe":case"object":case"embed":ge("load",r);break;case"video":case"audio":for(i=0;i<xs.length;i++)ge(xs[i],r);break;case"source":ge("error",r);break;case"img":case"image":case"link":ge("error",r),ge("load",r);break;case"details":ge("toggle",r);break;case"input":rp(r,s),ge("invalid",r);break;case"select":r._wrapperState={wasMultiple:!!s.multiple},ge("invalid",r);break;case"textarea":sp(r,s),ge("invalid",r)}Lu(n,s),i=null;for(var o in s)if(s.hasOwnProperty(o)){var l=s[o];o==="children"?typeof l=="string"?r.textContent!==l&&(s.suppressHydrationWarning!==!0&&Uo(r.textContent,l,t),i=["children",l]):typeof l=="number"&&r.textContent!==""+l&&(s.suppressHydrationWarning!==!0&&Uo(r.textContent,l,t),i=["children",""+l]):Vs.hasOwnProperty(o)&&l!=null&&o==="onScroll"&&ge("scroll",r)}switch(n){case"input":_o(r),ip(r,s,!0);break;case"textarea":_o(r),op(r);break;case"select":case"option":break;default:typeof s.onClick=="function"&&(r.onclick=Xl)}r=i,e.updateQueue=r,r!==null&&(e.flags|=4)}else{o=i.nodeType===9?i:i.ownerDocument,t==="http://www.w3.org/1999/xhtml"&&(t=v0(n)),t==="http://www.w3.org/1999/xhtml"?n==="script"?(t=o.createElement("div"),t.innerHTML="<script><\/script>",t=t.removeChild(t.firstChild)):typeof r.is=="string"?t=o.createElement(n,{is:r.is}):(t=o.createElement(n),n==="select"&&(o=t,r.multiple?o.multiple=!0:r.size&&(o.size=r.size))):t=o.createElementNS(t,n),t[un]=e,t[Zs]=r,zv(t,e,!1,!1),e.stateNode=t;e:{switch(o=Du(n,r),n){case"dialog":ge("cancel",t),ge("close",t),i=r;break;case"iframe":case"object":case"embed":ge("load",t),i=r;break;case"video":case"audio":for(i=0;i<xs.length;i++)ge(xs[i],t);i=r;break;case"source":ge("error",t),i=r;break;case"img":case"image":case"link":ge("error",t),ge("load",t),i=r;break;case"details":ge("toggle",t),i=r;break;case"input":rp(t,r),i=Mu(t,r),ge("invalid",t);break;case"option":i=r;break;case"select":t._wrapperState={wasMultiple:!!r.multiple},i=be({},r,{value:void 0}),ge("invalid",t);break;case"textarea":sp(t,r),i=Pu(t,r),ge("invalid",t);break;default:i=r}Lu(n,i),l=i;for(s in l)if(l.hasOwnProperty(s)){var a=l[s];s==="style"?S0(t,a):s==="dangerouslySetInnerHTML"?(a=a?a.__html:void 0,a!=null&&y0(t,a)):s==="children"?typeof a=="string"?(n!=="textarea"||a!=="")&&Hs(t,a):typeof a=="number"&&Hs(t,""+a):s!=="suppressContentEditableWarning"&&s!=="suppressHydrationWarning"&&s!=="autoFocus"&&(Vs.hasOwnProperty(s)?a!=null&&s==="onScroll"&&ge("scroll",t):a!=null&&vf(t,s,a,o))}switch(n){case"input":_o(t),ip(t,r,!1);break;case"textarea":_o(t),op(t);break;case"option":r.value!=null&&t.setAttribute("value",""+ar(r.value));break;case"select":t.multiple=!!r.multiple,s=r.value,s!=null?xi(t,!!r.multiple,s,!1):r.defaultValue!=null&&xi(t,!!r.multiple,r.defaultValue,!0);break;default:typeof i.onClick=="function"&&(t.onclick=Xl)}switch(n){case"button":case"input":case"select":case"textarea":r=!!r.autoFocus;break e;case"img":r=!0;break e;default:r=!1}}r&&(e.flags|=4)}e.ref!==null&&(e.flags|=512,e.flags|=2097152)}return Ye(e),null;case 6:if(t&&e.stateNode!=null)Vv(t,e,t.memoizedProps,r);else{if(typeof r!="string"&&e.stateNode===null)throw Error(E(166));if(n=Lr(to.current),Lr(Sn.current),$o(e)){if(r=e.stateNode,n=e.memoizedProps,r[un]=e,(s=r.nodeValue!==n)&&(t=Rt,t!==null))switch(t.tag){case 3:Uo(r.nodeValue,n,(t.mode&1)!==0);break;case 5:t.memoizedProps.suppressHydrationWarning!==!0&&Uo(r.nodeValue,n,(t.mode&1)!==0)}s&&(e.flags|=4)}else r=(n.nodeType===9?n:n.ownerDocument).createTextNode(r),r[un]=e,e.stateNode=r}return Ye(e),null;case 13:if(ve(xe),r=e.memoizedState,t===null||t.memoizedState!==null&&t.memoizedState.dehydrated!==null){if(ye&&bt!==null&&e.mode&1&&!(e.flags&128))lv(),_i(),e.flags|=98560,s=!1;else if(s=$o(e),r!==null&&r.dehydrated!==null){if(t===null){if(!s)throw Error(E(318));if(s=e.memoizedState,s=s!==null?s.dehydrated:null,!s)throw Error(E(317));s[un]=e}else _i(),!(e.flags&128)&&(e.memoizedState=null),e.flags|=4;Ye(e),s=!1}else Ut!==null&&(dh(Ut),Ut=null),s=!0;if(!s)return e.flags&65536?e:null}return e.flags&128?(e.lanes=n,e):(r=r!==null,r!==(t!==null&&t.memoizedState!==null)&&r&&(e.child.flags|=8192,e.mode&1&&(t===null||xe.current&1?Oe===0&&(Oe=3):Zf())),e.updateQueue!==null&&(e.flags|=4),Ye(e),null);case 4:return Wi(),sh(t,e),t===null&&Xs(e.stateNode.containerInfo),Ye(e),null;case 10:return Bf(e.type._context),Ye(e),null;case 17:return yt(e.type)&&Jl(),Ye(e),null;case 19:if(ve(xe),s=e.memoizedState,s===null)return Ye(e),null;if(r=(e.flags&128)!==0,o=s.rendering,o===null)if(r)hs(s,!1);else{if(Oe!==0||t!==null&&t.flags&128)for(t=e.child;t!==null;){if(o=sa(t),o!==null){for(e.flags|=128,hs(s,!1),r=o.updateQueue,r!==null&&(e.updateQueue=r,e.flags|=4),e.subtreeFlags=0,r=n,n=e.child;n!==null;)s=n,t=r,s.flags&=14680066,o=s.alternate,o===null?(s.childLanes=0,s.lanes=t,s.child=null,s.subtreeFlags=0,s.memoizedProps=null,s.memoizedState=null,s.updateQueue=null,s.dependencies=null,s.stateNode=null):(s.childLanes=o.childLanes,s.lanes=o.lanes,s.child=o.child,s.subtreeFlags=0,s.deletions=null,s.memoizedProps=o.memoizedProps,s.memoizedState=o.memoizedState,s.updateQueue=o.updateQueue,s.type=o.type,t=o.dependencies,s.dependencies=t===null?null:{lanes:t.lanes,firstContext:t.firstContext}),n=n.sibling;return fe(xe,xe.current&1|2),e.child}t=t.sibling}s.tail!==null&&Pe()>zi&&(e.flags|=128,r=!0,hs(s,!1),e.lanes=4194304)}else{if(!r)if(t=sa(o),t!==null){if(e.flags|=128,r=!0,n=t.updateQueue,n!==null&&(e.updateQueue=n,e.flags|=4),hs(s,!0),s.tail===null&&s.tailMode==="hidden"&&!o.alternate&&!ye)return Ye(e),null}else 2*Pe()-s.renderingStartTime>zi&&n!==1073741824&&(e.flags|=128,r=!0,hs(s,!1),e.lanes=4194304);s.isBackwards?(o.sibling=e.child,e.child=o):(n=s.last,n!==null?n.sibling=o:e.child=o,s.last=o)}return s.tail!==null?(e=s.tail,s.rendering=e,s.tail=e.sibling,s.renderingStartTime=Pe(),e.sibling=null,n=xe.current,fe(xe,r?n&1|2:n&1),e):(Ye(e),null);case 22:case 23:return Jf(),r=e.memoizedState!==null,t!==null&&t.memoizedState!==null!==r&&(e.flags|=8192),r&&e.mode&1?xt&1073741824&&(Ye(e),e.subtreeFlags&6&&(e.flags|=8192)):Ye(e),null;case 24:return null;case 25:return null}throw Error(E(156,e.tag))}function Wx(t,e){switch(Lf(e),e.tag){case 1:return yt(e.type)&&Jl(),t=e.flags,t&65536?(e.flags=t&-65537|128,e):null;case 3:return Wi(),ve(vt),ve(Ze),zf(),t=e.flags,t&65536&&!(t&128)?(e.flags=t&-65537|128,e):null;case 5:return Ff(e),null;case 13:if(ve(xe),t=e.memoizedState,t!==null&&t.dehydrated!==null){if(e.alternate===null)throw Error(E(340));_i()}return t=e.flags,t&65536?(e.flags=t&-65537|128,e):null;case 19:return ve(xe),null;case 4:return Wi(),null;case 10:return Bf(e.type._context),null;case 22:case 23:return Jf(),null;case 24:return null;default:return null}}var Qo=!1,Je=!1,Fx=typeof WeakSet=="function"?WeakSet:Set,V=null;function gi(t,e){var n=t.ref;if(n!==null)if(typeof n=="function")try{n(null)}catch(r){Te(t,e,r)}else n.current=null}function oh(t,e,n){try{n()}catch(r){Te(t,e,r)}}var Qp=!1;function zx(t,e){if(Vu=Kl,t=K0(),Pf(t)){if("selectionStart"in t)var n={start:t.selectionStart,end:t.selectionEnd};else e:{n=(n=t.ownerDocument)&&n.defaultView||window;var r=n.getSelection&&n.getSelection();if(r&&r.rangeCount!==0){n=r.anchorNode;var i=r.anchorOffset,s=r.focusNode;r=r.focusOffset;try{n.nodeType,s.nodeType}catch{n=null;break e}var o=0,l=-1,a=-1,c=0,u=0,h=t,f=null;t:for(;;){for(var d;h!==n||i!==0&&h.nodeType!==3||(l=o+i),h!==s||r!==0&&h.nodeType!==3||(a=o+r),h.nodeType===3&&(o+=h.nodeValue.length),(d=h.firstChild)!==null;)f=h,h=d;for(;;){if(h===t)break t;if(f===n&&++c===i&&(l=o),f===s&&++u===r&&(a=o),(d=h.nextSibling)!==null)break;h=f,f=h.parentNode}h=d}n=l===-1||a===-1?null:{start:l,end:a}}else n=null}n=n||{start:0,end:0}}else n=null;for(Hu={focusedElem:t,selectionRange:n},Kl=!1,V=e;V!==null;)if(e=V,t=e.child,(e.subtreeFlags&1028)!==0&&t!==null)t.return=e,V=t;else for(;V!==null;){e=V;try{var p=e.alternate;if(e.flags&1024)switch(e.tag){case 0:case 11:case 15:break;case 1:if(p!==null){var m=p.memoizedProps,w=p.memoizedState,g=e.stateNode,v=g.getSnapshotBeforeUpdate(e.elementType===e.type?m:Vt(e.type,m),w);g.__reactInternalSnapshotBeforeUpdate=v}break;case 3:var y=e.stateNode.containerInfo;y.nodeType===1?y.textContent="":y.nodeType===9&&y.documentElement&&y.removeChild(y.documentElement);break;case 5:case 6:case 4:case 17:break;default:throw Error(E(163))}}catch(x){Te(e,e.return,x)}if(t=e.sibling,t!==null){t.return=e.return,V=t;break}V=e.return}return p=Qp,Qp=!1,p}function Ls(t,e,n){var r=e.updateQueue;if(r=r!==null?r.lastEffect:null,r!==null){var i=r=r.next;do{if((i.tag&t)===t){var s=i.destroy;i.destroy=void 0,s!==void 0&&oh(e,n,s)}i=i.next}while(i!==r)}}function Ha(t,e){if(e=e.updateQueue,e=e!==null?e.lastEffect:null,e!==null){var n=e=e.next;do{if((n.tag&t)===t){var r=n.create;n.destroy=r()}n=n.next}while(n!==e)}}function lh(t){var e=t.ref;if(e!==null){var n=t.stateNode;switch(t.tag){case 5:t=n;break;default:t=n}typeof e=="function"?e(t):e.current=t}}function Hv(t){var e=t.alternate;e!==null&&(t.alternate=null,Hv(e)),t.child=null,t.deletions=null,t.sibling=null,t.tag===5&&(e=t.stateNode,e!==null&&(delete e[un],delete e[Zs],delete e[Gu],delete e[Cx],delete e[bx])),t.stateNode=null,t.return=null,t.dependencies=null,t.memoizedProps=null,t.memoizedState=null,t.pendingProps=null,t.stateNode=null,t.updateQueue=null}function Uv(t){return t.tag===5||t.tag===3||t.tag===4}function Yp(t){e:for(;;){for(;t.sibling===null;){if(t.return===null||Uv(t.return))return null;t=t.return}for(t.sibling.return=t.return,t=t.sibling;t.tag!==5&&t.tag!==6&&t.tag!==18;){if(t.flags&2||t.child===null||t.tag===4)continue e;t.child.return=t,t=t.child}if(!(t.flags&2))return t.stateNode}}function ah(t,e,n){var r=t.tag;if(r===5||r===6)t=t.stateNode,e?n.nodeType===8?n.parentNode.insertBefore(t,e):n.insertBefore(t,e):(n.nodeType===8?(e=n.parentNode,e.insertBefore(t,n)):(e=n,e.appendChild(t)),n=n._reactRootContainer,n!=null||e.onclick!==null||(e.onclick=Xl));else if(r!==4&&(t=t.child,t!==null))for(ah(t,e,n),t=t.sibling;t!==null;)ah(t,e,n),t=t.sibling}function ch(t,e,n){var r=t.tag;if(r===5||r===6)t=t.stateNode,e?n.insertBefore(t,e):n.appendChild(t);else if(r!==4&&(t=t.child,t!==null))for(ch(t,e,n),t=t.sibling;t!==null;)ch(t,e,n),t=t.sibling}var Ve=null,Ht=!1;function Wn(t,e,n){for(n=n.child;n!==null;)$v(t,e,n),n=n.sibling}function $v(t,e,n){if(wn&&typeof wn.onCommitFiberUnmount=="function")try{wn.onCommitFiberUnmount(Ba,n)}catch{}switch(n.tag){case 5:Je||gi(n,e);case 6:var r=Ve,i=Ht;Ve=null,Wn(t,e,n),Ve=r,Ht=i,Ve!==null&&(Ht?(t=Ve,n=n.stateNode,t.nodeType===8?t.parentNode.removeChild(n):t.removeChild(n)):Ve.removeChild(n.stateNode));break;case 18:Ve!==null&&(Ht?(t=Ve,n=n.stateNode,t.nodeType===8?Oc(t.parentNode,n):t.nodeType===1&&Oc(t,n),Ks(t)):Oc(Ve,n.stateNode));break;case 4:r=Ve,i=Ht,Ve=n.stateNode.containerInfo,Ht=!0,Wn(t,e,n),Ve=r,Ht=i;break;case 0:case 11:case 14:case 15:if(!Je&&(r=n.updateQueue,r!==null&&(r=r.lastEffect,r!==null))){i=r=r.next;do{var s=i,o=s.destroy;s=s.tag,o!==void 0&&(s&2||s&4)&&oh(n,e,o),i=i.next}while(i!==r)}Wn(t,e,n);break;case 1:if(!Je&&(gi(n,e),r=n.stateNode,typeof r.componentWillUnmount=="function"))try{r.props=n.memoizedProps,r.state=n.memoizedState,r.componentWillUnmount()}catch(l){Te(n,e,l)}Wn(t,e,n);break;case 21:Wn(t,e,n);break;case 22:n.mode&1?(Je=(r=Je)||n.memoizedState!==null,Wn(t,e,n),Je=r):Wn(t,e,n);break;default:Wn(t,e,n)}}function Xp(t){var e=t.updateQueue;if(e!==null){t.updateQueue=null;var n=t.stateNode;n===null&&(n=t.stateNode=new Fx),e.forEach(function(r){var i=Yx.bind(null,t,r);n.has(r)||(n.add(r),r.then(i,i))})}}function jt(t,e){var n=e.deletions;if(n!==null)for(var r=0;r<n.length;r++){var i=n[r];try{var s=t,o=e,l=o;e:for(;l!==null;){switch(l.tag){case 5:Ve=l.stateNode,Ht=!1;break e;case 3:Ve=l.stateNode.containerInfo,Ht=!0;break e;case 4:Ve=l.stateNode.containerInfo,Ht=!0;break e}l=l.return}if(Ve===null)throw Error(E(160));$v(s,o,i),Ve=null,Ht=!1;var a=i.alternate;a!==null&&(a.return=null),i.return=null}catch(c){Te(i,e,c)}}if(e.subtreeFlags&12854)for(e=e.child;e!==null;)Gv(e,t),e=e.sibling}function Gv(t,e){var n=t.alternate,r=t.flags;switch(t.tag){case 0:case 11:case 14:case 15:if(jt(e,t),nn(t),r&4){try{Ls(3,t,t.return),Ha(3,t)}catch(m){Te(t,t.return,m)}try{Ls(5,t,t.return)}catch(m){Te(t,t.return,m)}}break;case 1:jt(e,t),nn(t),r&512&&n!==null&&gi(n,n.return);break;case 5:if(jt(e,t),nn(t),r&512&&n!==null&&gi(n,n.return),t.flags&32){var i=t.stateNode;try{Hs(i,"")}catch(m){Te(t,t.return,m)}}if(r&4&&(i=t.stateNode,i!=null)){var s=t.memoizedProps,o=n!==null?n.memoizedProps:s,l=t.type,a=t.updateQueue;if(t.updateQueue=null,a!==null)try{l==="input"&&s.type==="radio"&&s.name!=null&&m0(i,s),Du(l,o);var c=Du(l,s);for(o=0;o<a.length;o+=2){var u=a[o],h=a[o+1];u==="style"?S0(i,h):u==="dangerouslySetInnerHTML"?y0(i,h):u==="children"?Hs(i,h):vf(i,u,h,c)}switch(l){case"input":Iu(i,s);break;case"textarea":g0(i,s);break;case"select":var f=i._wrapperState.wasMultiple;i._wrapperState.wasMultiple=!!s.multiple;var d=s.value;d!=null?xi(i,!!s.multiple,d,!1):f!==!!s.multiple&&(s.defaultValue!=null?xi(i,!!s.multiple,s.defaultValue,!0):xi(i,!!s.multiple,s.multiple?[]:"",!1))}i[Zs]=s}catch(m){Te(t,t.return,m)}}break;case 6:if(jt(e,t),nn(t),r&4){if(t.stateNode===null)throw Error(E(162));i=t.stateNode,s=t.memoizedProps;try{i.nodeValue=s}catch(m){Te(t,t.return,m)}}break;case 3:if(jt(e,t),nn(t),r&4&&n!==null&&n.memoizedState.isDehydrated)try{Ks(e.containerInfo)}catch(m){Te(t,t.return,m)}break;case 4:jt(e,t),nn(t);break;case 13:jt(e,t),nn(t),i=t.child,i.flags&8192&&(s=i.memoizedState!==null,i.stateNode.isHidden=s,!s||i.alternate!==null&&i.alternate.memoizedState!==null||(Yf=Pe())),r&4&&Xp(t);break;case 22:if(u=n!==null&&n.memoizedState!==null,t.mode&1?(Je=(c=Je)||u,jt(e,t),Je=c):jt(e,t),nn(t),r&8192){if(c=t.memoizedState!==null,(t.stateNode.isHidden=c)&&!u&&t.mode&1)for(V=t,u=t.child;u!==null;){for(h=V=u;V!==null;){switch(f=V,d=f.child,f.tag){case 0:case 11:case 14:case 15:Ls(4,f,f.return);break;case 1:gi(f,f.return);var p=f.stateNode;if(typeof p.componentWillUnmount=="function"){r=f,n=f.return;try{e=r,p.props=e.memoizedProps,p.state=e.memoizedState,p.componentWillUnmount()}catch(m){Te(r,n,m)}}break;case 5:gi(f,f.return);break;case 22:if(f.memoizedState!==null){Zp(h);continue}}d!==null?(d.return=f,V=d):Zp(h)}u=u.sibling}e:for(u=null,h=t;;){if(h.tag===5){if(u===null){u=h;try{i=h.stateNode,c?(s=i.style,typeof s.setProperty=="function"?s.setProperty("display","none","important"):s.display="none"):(l=h.stateNode,a=h.memoizedProps.style,o=a!=null&&a.hasOwnProperty("display")?a.display:null,l.style.display=w0("display",o))}catch(m){Te(t,t.return,m)}}}else if(h.tag===6){if(u===null)try{h.stateNode.nodeValue=c?"":h.memoizedProps}catch(m){Te(t,t.return,m)}}else if((h.tag!==22&&h.tag!==23||h.memoizedState===null||h===t)&&h.child!==null){h.child.return=h,h=h.child;continue}if(h===t)break e;for(;h.sibling===null;){if(h.return===null||h.return===t)break e;u===h&&(u=null),h=h.return}u===h&&(u=null),h.sibling.return=h.return,h=h.sibling}}break;case 19:jt(e,t),nn(t),r&4&&Xp(t);break;case 21:break;default:jt(e,t),nn(t)}}function nn(t){var e=t.flags;if(e&2){try{e:{for(var n=t.return;n!==null;){if(Uv(n)){var r=n;break e}n=n.return}throw Error(E(160))}switch(r.tag){case 5:var i=r.stateNode;r.flags&32&&(Hs(i,""),r.flags&=-33);var s=Yp(t);ch(t,s,i);break;case 3:case 4:var o=r.stateNode.containerInfo,l=Yp(t);ah(t,l,o);break;default:throw Error(E(161))}}catch(a){Te(t,t.return,a)}t.flags&=-3}e&4096&&(t.flags&=-4097)}function jx(t,e,n){V=t,Kv(t)}function Kv(t,e,n){for(var r=(t.mode&1)!==0;V!==null;){var i=V,s=i.child;if(i.tag===22&&r){var o=i.memoizedState!==null||Qo;if(!o){var l=i.alternate,a=l!==null&&l.memoizedState!==null||Je;l=Qo;var c=Je;if(Qo=o,(Je=a)&&!c)for(V=i;V!==null;)o=V,a=o.child,o.tag===22&&o.memoizedState!==null?em(i):a!==null?(a.return=o,V=a):em(i);for(;s!==null;)V=s,Kv(s),s=s.sibling;V=i,Qo=l,Je=c}Jp(t)}else i.subtreeFlags&8772&&s!==null?(s.return=i,V=s):Jp(t)}}function Jp(t){for(;V!==null;){var e=V;if(e.flags&8772){var n=e.alternate;try{if(e.flags&8772)switch(e.tag){case 0:case 11:case 15:Je||Ha(5,e);break;case 1:var r=e.stateNode;if(e.flags&4&&!Je)if(n===null)r.componentDidMount();else{var i=e.elementType===e.type?n.memoizedProps:Vt(e.type,n.memoizedProps);r.componentDidUpdate(i,n.memoizedState,r.__reactInternalSnapshotBeforeUpdate)}var s=e.updateQueue;s!==null&&Bp(e,s,r);break;case 3:var o=e.updateQueue;if(o!==null){if(n=null,e.child!==null)switch(e.child.tag){case 5:n=e.child.stateNode;break;case 1:n=e.child.stateNode}Bp(e,o,n)}break;case 5:var l=e.stateNode;if(n===null&&e.flags&4){n=l;var a=e.memoizedProps;switch(e.type){case"button":case"input":case"select":case"textarea":a.autoFocus&&n.focus();break;case"img":a.src&&(n.src=a.src)}}break;case 6:break;case 4:break;case 12:break;case 13:if(e.memoizedState===null){var c=e.alternate;if(c!==null){var u=c.memoizedState;if(u!==null){var h=u.dehydrated;h!==null&&Ks(h)}}}break;case 19:case 17:case 21:case 22:case 23:case 25:break;default:throw Error(E(163))}Je||e.flags&512&&lh(e)}catch(f){Te(e,e.return,f)}}if(e===t){V=null;break}if(n=e.sibling,n!==null){n.return=e.return,V=n;break}V=e.return}}function Zp(t){for(;V!==null;){var e=V;if(e===t){V=null;break}var n=e.sibling;if(n!==null){n.return=e.return,V=n;break}V=e.return}}function em(t){for(;V!==null;){var e=V;try{switch(e.tag){case 0:case 11:case 15:var n=e.return;try{Ha(4,e)}catch(a){Te(e,n,a)}break;case 1:var r=e.stateNode;if(typeof r.componentDidMount=="function"){var i=e.return;try{r.componentDidMount()}catch(a){Te(e,i,a)}}var s=e.return;try{lh(e)}catch(a){Te(e,s,a)}break;case 5:var o=e.return;try{lh(e)}catch(a){Te(e,o,a)}}}catch(a){Te(e,e.return,a)}if(e===t){V=null;break}var l=e.sibling;if(l!==null){l.return=e.return,V=l;break}V=e.return}}var Vx=Math.ceil,aa=Bn.ReactCurrentDispatcher,Kf=Bn.ReactCurrentOwner,_t=Bn.ReactCurrentBatchConfig,ne=0,We=null,Ee=null,$e=0,xt=0,vi=yr(0),Oe=0,so=null,Vr=0,Ua=0,Qf=0,Ds=null,ht=null,Yf=0,zi=1/0,bn=null,ca=!1,uh=null,ir=null,Yo=!1,Yn=null,ua=0,Os=0,hh=null,Pl=-1,El=0;function ot(){return ne&6?Pe():Pl!==-1?Pl:Pl=Pe()}function sr(t){return t.mode&1?ne&2&&$e!==0?$e&-$e:Rx.transition!==null?(El===0&&(El=L0()),El):(t=oe,t!==0||(t=window.event,t=t===void 0?16:W0(t.type)),t):1}function Yt(t,e,n,r){if(50<Os)throw Os=0,hh=null,Error(E(185));Co(t,n,r),(!(ne&2)||t!==We)&&(t===We&&(!(ne&2)&&(Ua|=n),Oe===4&&Un(t,$e)),wt(t,r),n===1&&ne===0&&!(e.mode&1)&&(zi=Pe()+500,za&&wr()))}function wt(t,e){var n=t.callbackNode;Rk(t,e);var r=Gl(t,t===We?$e:0);if(r===0)n!==null&&cp(n),t.callbackNode=null,t.callbackPriority=0;else if(e=r&-r,t.callbackPriority!==e){if(n!=null&&cp(n),e===1)t.tag===0?Tx(tm.bind(null,t)):iv(tm.bind(null,t)),kx(function(){!(ne&6)&&wr()}),n=null;else{switch(D0(r)){case 1:n=xf;break;case 4:n=P0;break;case 16:n=$l;break;case 536870912:n=E0;break;default:n=$l}n=ny(n,Qv.bind(null,t))}t.callbackPriority=e,t.callbackNode=n}}function Qv(t,e){if(Pl=-1,El=0,ne&6)throw Error(E(327));var n=t.callbackNode;if(Mi()&&t.callbackNode!==n)return null;var r=Gl(t,t===We?$e:0);if(r===0)return null;if(r&30||r&t.expiredLanes||e)e=ha(t,r);else{e=r;var i=ne;ne|=2;var s=Xv();(We!==t||$e!==e)&&(bn=null,zi=Pe()+500,Nr(t,e));do try{$x();break}catch(l){Yv(t,l)}while(!0);Nf(),aa.current=s,ne=i,Ee!==null?e=0:(We=null,$e=0,e=Oe)}if(e!==0){if(e===2&&(i=qu(t),i!==0&&(r=i,e=fh(t,i))),e===1)throw n=so,Nr(t,0),Un(t,r),wt(t,Pe()),n;if(e===6)Un(t,r);else{if(i=t.current.alternate,!(r&30)&&!Hx(i)&&(e=ha(t,r),e===2&&(s=qu(t),s!==0&&(r=s,e=fh(t,s))),e===1))throw n=so,Nr(t,0),Un(t,r),wt(t,Pe()),n;switch(t.finishedWork=i,t.finishedLanes=r,e){case 0:case 1:throw Error(E(345));case 2:Rr(t,ht,bn);break;case 3:if(Un(t,r),(r&130023424)===r&&(e=Yf+500-Pe(),10<e)){if(Gl(t,0)!==0)break;if(i=t.suspendedLanes,(i&r)!==r){ot(),t.pingedLanes|=t.suspendedLanes&i;break}t.timeoutHandle=$u(Rr.bind(null,t,ht,bn),e);break}Rr(t,ht,bn);break;case 4:if(Un(t,r),(r&4194240)===r)break;for(e=t.eventTimes,i=-1;0<r;){var o=31-Qt(r);s=1<<o,o=e[o],o>i&&(i=o),r&=~s}if(r=i,r=Pe()-r,r=(120>r?120:480>r?480:1080>r?1080:1920>r?1920:3e3>r?3e3:4320>r?4320:1960*Vx(r/1960))-r,10<r){t.timeoutHandle=$u(Rr.bind(null,t,ht,bn),r);break}Rr(t,ht,bn);break;case 5:Rr(t,ht,bn);break;default:throw Error(E(329))}}}return wt(t,Pe()),t.callbackNode===n?Qv.bind(null,t):null}function fh(t,e){var n=Ds;return t.current.memoizedState.isDehydrated&&(Nr(t,e).flags|=256),t=ha(t,e),t!==2&&(e=ht,ht=n,e!==null&&dh(e)),t}function dh(t){ht===null?ht=t:ht.push.apply(ht,t)}function Hx(t){for(var e=t;;){if(e.flags&16384){var n=e.updateQueue;if(n!==null&&(n=n.stores,n!==null))for(var r=0;r<n.length;r++){var i=n[r],s=i.getSnapshot;i=i.value;try{if(!Xt(s(),i))return!1}catch{return!1}}}if(n=e.child,e.subtreeFlags&16384&&n!==null)n.return=e,e=n;else{if(e===t)break;for(;e.sibling===null;){if(e.return===null||e.return===t)return!0;e=e.return}e.sibling.return=e.return,e=e.sibling}}return!0}function Un(t,e){for(e&=~Qf,e&=~Ua,t.suspendedLanes|=e,t.pingedLanes&=~e,t=t.expirationTimes;0<e;){var n=31-Qt(e),r=1<<n;t[n]=-1,e&=~r}}function tm(t){if(ne&6)throw Error(E(327));Mi();var e=Gl(t,0);if(!(e&1))return wt(t,Pe()),null;var n=ha(t,e);if(t.tag!==0&&n===2){var r=qu(t);r!==0&&(e=r,n=fh(t,r))}if(n===1)throw n=so,Nr(t,0),Un(t,e),wt(t,Pe()),n;if(n===6)throw Error(E(345));return t.finishedWork=t.current.alternate,t.finishedLanes=e,Rr(t,ht,bn),wt(t,Pe()),null}function Xf(t,e){var n=ne;ne|=1;try{return t(e)}finally{ne=n,ne===0&&(zi=Pe()+500,za&&wr())}}function Hr(t){Yn!==null&&Yn.tag===0&&!(ne&6)&&Mi();var e=ne;ne|=1;var n=_t.transition,r=oe;try{if(_t.transition=null,oe=1,t)return t()}finally{oe=r,_t.transition=n,ne=e,!(ne&6)&&wr()}}function Jf(){xt=vi.current,ve(vi)}function Nr(t,e){t.finishedWork=null,t.finishedLanes=0;var n=t.timeoutHandle;if(n!==-1&&(t.timeoutHandle=-1,Sx(n)),Ee!==null)for(n=Ee.return;n!==null;){var r=n;switch(Lf(r),r.tag){case 1:r=r.type.childContextTypes,r!=null&&Jl();break;case 3:Wi(),ve(vt),ve(Ze),zf();break;case 5:Ff(r);break;case 4:Wi();break;case 13:ve(xe);break;case 19:ve(xe);break;case 10:Bf(r.type._context);break;case 22:case 23:Jf()}n=n.return}if(We=t,Ee=t=or(t.current,null),$e=xt=e,Oe=0,so=null,Qf=Ua=Vr=0,ht=Ds=null,Er!==null){for(e=0;e<Er.length;e++)if(n=Er[e],r=n.interleaved,r!==null){n.interleaved=null;var i=r.next,s=n.pending;if(s!==null){var o=s.next;s.next=i,r.next=o}n.pending=r}Er=null}return t}function Yv(t,e){do{var n=Ee;try{if(Nf(),Ml.current=la,oa){for(var r=Ce.memoizedState;r!==null;){var i=r.queue;i!==null&&(i.pending=null),r=r.next}oa=!1}if(jr=0,Be=Le=Ce=null,Es=!1,no=0,Kf.current=null,n===null||n.return===null){Oe=1,so=e,Ee=null;break}e:{var s=t,o=n.return,l=n,a=e;if(e=$e,l.flags|=32768,a!==null&&typeof a=="object"&&typeof a.then=="function"){var c=a,u=l,h=u.tag;if(!(u.mode&1)&&(h===0||h===11||h===15)){var f=u.alternate;f?(u.updateQueue=f.updateQueue,u.memoizedState=f.memoizedState,u.lanes=f.lanes):(u.updateQueue=null,u.memoizedState=null)}var d=jp(o);if(d!==null){d.flags&=-257,Vp(d,o,l,s,e),d.mode&1&&zp(s,c,e),e=d,a=c;var p=e.updateQueue;if(p===null){var m=new Set;m.add(a),e.updateQueue=m}else p.add(a);break e}else{if(!(e&1)){zp(s,c,e),Zf();break e}a=Error(E(426))}}else if(ye&&l.mode&1){var w=jp(o);if(w!==null){!(w.flags&65536)&&(w.flags|=256),Vp(w,o,l,s,e),Df(Fi(a,l));break e}}s=a=Fi(a,l),Oe!==4&&(Oe=2),Ds===null?Ds=[s]:Ds.push(s),s=o;do{switch(s.tag){case 3:s.flags|=65536,e&=-e,s.lanes|=e;var g=Dv(s,a,e);Np(s,g);break e;case 1:l=a;var v=s.type,y=s.stateNode;if(!(s.flags&128)&&(typeof v.getDerivedStateFromError=="function"||y!==null&&typeof y.componentDidCatch=="function"&&(ir===null||!ir.has(y)))){s.flags|=65536,e&=-e,s.lanes|=e;var x=Ov(s,l,e);Np(s,x);break e}}s=s.return}while(s!==null)}Zv(n)}catch(k){e=k,Ee===n&&n!==null&&(Ee=n=n.return);continue}break}while(!0)}function Xv(){var t=aa.current;return aa.current=la,t===null?la:t}function Zf(){(Oe===0||Oe===3||Oe===2)&&(Oe=4),We===null||!(Vr&268435455)&&!(Ua&268435455)||Un(We,$e)}function ha(t,e){var n=ne;ne|=2;var r=Xv();(We!==t||$e!==e)&&(bn=null,Nr(t,e));do try{Ux();break}catch(i){Yv(t,i)}while(!0);if(Nf(),ne=n,aa.current=r,Ee!==null)throw Error(E(261));return We=null,$e=0,Oe}function Ux(){for(;Ee!==null;)Jv(Ee)}function $x(){for(;Ee!==null&&!vk();)Jv(Ee)}function Jv(t){var e=ty(t.alternate,t,xt);t.memoizedProps=t.pendingProps,e===null?Zv(t):Ee=e,Kf.current=null}function Zv(t){var e=t;do{var n=e.alternate;if(t=e.return,e.flags&32768){if(n=Wx(n,e),n!==null){n.flags&=32767,Ee=n;return}if(t!==null)t.flags|=32768,t.subtreeFlags=0,t.deletions=null;else{Oe=6,Ee=null;return}}else if(n=qx(n,e,xt),n!==null){Ee=n;return}if(e=e.sibling,e!==null){Ee=e;return}Ee=e=t}while(e!==null);Oe===0&&(Oe=5)}function Rr(t,e,n){var r=oe,i=_t.transition;try{_t.transition=null,oe=1,Gx(t,e,n,r)}finally{_t.transition=i,oe=r}return null}function Gx(t,e,n,r){do Mi();while(Yn!==null);if(ne&6)throw Error(E(327));n=t.finishedWork;var i=t.finishedLanes;if(n===null)return null;if(t.finishedWork=null,t.finishedLanes=0,n===t.current)throw Error(E(177));t.callbackNode=null,t.callbackPriority=0;var s=n.lanes|n.childLanes;if(Mk(t,s),t===We&&(Ee=We=null,$e=0),!(n.subtreeFlags&2064)&&!(n.flags&2064)||Yo||(Yo=!0,ny($l,function(){return Mi(),null})),s=(n.flags&15990)!==0,n.subtreeFlags&15990||s){s=_t.transition,_t.transition=null;var o=oe;oe=1;var l=ne;ne|=4,Kf.current=null,zx(t,n),Gv(n,t),dx(Hu),Kl=!!Vu,Hu=Vu=null,t.current=n,jx(n),yk(),ne=l,oe=o,_t.transition=s}else t.current=n;if(Yo&&(Yo=!1,Yn=t,ua=i),s=t.pendingLanes,s===0&&(ir=null),kk(n.stateNode),wt(t,Pe()),e!==null)for(r=t.onRecoverableError,n=0;n<e.length;n++)i=e[n],r(i.value,{componentStack:i.stack,digest:i.digest});if(ca)throw ca=!1,t=uh,uh=null,t;return ua&1&&t.tag!==0&&Mi(),s=t.pendingLanes,s&1?t===hh?Os++:(Os=0,hh=t):Os=0,wr(),null}function Mi(){if(Yn!==null){var t=D0(ua),e=_t.transition,n=oe;try{if(_t.transition=null,oe=16>t?16:t,Yn===null)var r=!1;else{if(t=Yn,Yn=null,ua=0,ne&6)throw Error(E(331));var i=ne;for(ne|=4,V=t.current;V!==null;){var s=V,o=s.child;if(V.flags&16){var l=s.deletions;if(l!==null){for(var a=0;a<l.length;a++){var c=l[a];for(V=c;V!==null;){var u=V;switch(u.tag){case 0:case 11:case 15:Ls(8,u,s)}var h=u.child;if(h!==null)h.return=u,V=h;else for(;V!==null;){u=V;var f=u.sibling,d=u.return;if(Hv(u),u===c){V=null;break}if(f!==null){f.return=d,V=f;break}V=d}}}var p=s.alternate;if(p!==null){var m=p.child;if(m!==null){p.child=null;do{var w=m.sibling;m.sibling=null,m=w}while(m!==null)}}V=s}}if(s.subtreeFlags&2064&&o!==null)o.return=s,V=o;else e:for(;V!==null;){if(s=V,s.flags&2048)switch(s.tag){case 0:case 11:case 15:Ls(9,s,s.return)}var g=s.sibling;if(g!==null){g.return=s.return,V=g;break e}V=s.return}}var v=t.current;for(V=v;V!==null;){o=V;var y=o.child;if(o.subtreeFlags&2064&&y!==null)y.return=o,V=y;else e:for(o=v;V!==null;){if(l=V,l.flags&2048)try{switch(l.tag){case 0:case 11:case 15:Ha(9,l)}}catch(k){Te(l,l.return,k)}if(l===o){V=null;break e}var x=l.sibling;if(x!==null){x.return=l.return,V=x;break e}V=l.return}}if(ne=i,wr(),wn&&typeof wn.onPostCommitFiberRoot=="function")try{wn.onPostCommitFiberRoot(Ba,t)}catch{}r=!0}return r}finally{oe=n,_t.transition=e}}return!1}function nm(t,e,n){e=Fi(n,e),e=Dv(t,e,1),t=rr(t,e,1),e=ot(),t!==null&&(Co(t,1,e),wt(t,e))}function Te(t,e,n){if(t.tag===3)nm(t,t,n);else for(;e!==null;){if(e.tag===3){nm(e,t,n);break}else if(e.tag===1){var r=e.stateNode;if(typeof e.type.getDerivedStateFromError=="function"||typeof r.componentDidCatch=="function"&&(ir===null||!ir.has(r))){t=Fi(n,t),t=Ov(e,t,1),e=rr(e,t,1),t=ot(),e!==null&&(Co(e,1,t),wt(e,t));break}}e=e.return}}function Kx(t,e,n){var r=t.pingCache;r!==null&&r.delete(e),e=ot(),t.pingedLanes|=t.suspendedLanes&n,We===t&&($e&n)===n&&(Oe===4||Oe===3&&($e&130023424)===$e&&500>Pe()-Yf?Nr(t,0):Qf|=n),wt(t,e)}function ey(t,e){e===0&&(t.mode&1?(e=Fo,Fo<<=1,!(Fo&130023424)&&(Fo=4194304)):e=1);var n=ot();t=Dn(t,e),t!==null&&(Co(t,e,n),wt(t,n))}function Qx(t){var e=t.memoizedState,n=0;e!==null&&(n=e.retryLane),ey(t,n)}function Yx(t,e){var n=0;switch(t.tag){case 13:var r=t.stateNode,i=t.memoizedState;i!==null&&(n=i.retryLane);break;case 19:r=t.stateNode;break;default:throw Error(E(314))}r!==null&&r.delete(e),ey(t,n)}var ty;ty=function(t,e,n){if(t!==null)if(t.memoizedProps!==e.pendingProps||vt.current)mt=!0;else{if(!(t.lanes&n)&&!(e.flags&128))return mt=!1,_x(t,e,n);mt=!!(t.flags&131072)}else mt=!1,ye&&e.flags&1048576&&sv(e,ta,e.index);switch(e.lanes=0,e.tag){case 2:var r=e.type;Al(t,e),t=e.pendingProps;var i=Bi(e,Ze.current);Ri(e,n),i=Vf(null,e,r,t,i,n);var s=Hf();return e.flags|=1,typeof i=="object"&&i!==null&&typeof i.render=="function"&&i.$$typeof===void 0?(e.tag=1,e.memoizedState=null,e.updateQueue=null,yt(r)?(s=!0,Zl(e)):s=!1,e.memoizedState=i.state!==null&&i.state!==void 0?i.state:null,qf(e),i.updater=Va,e.stateNode=i,i._reactInternals=e,Zu(e,r,t,n),e=nh(null,e,r,!0,s,n)):(e.tag=0,ye&&s&&Ef(e),nt(null,e,i,n),e=e.child),e;case 16:r=e.elementType;e:{switch(Al(t,e),t=e.pendingProps,i=r._init,r=i(r._payload),e.type=r,i=e.tag=Jx(r),t=Vt(r,t),i){case 0:e=th(null,e,r,t,n);break e;case 1:e=$p(null,e,r,t,n);break e;case 11:e=Hp(null,e,r,t,n);break e;case 14:e=Up(null,e,r,Vt(r.type,t),n);break e}throw Error(E(306,r,""))}return e;case 0:return r=e.type,i=e.pendingProps,i=e.elementType===r?i:Vt(r,i),th(t,e,r,i,n);case 1:return r=e.type,i=e.pendingProps,i=e.elementType===r?i:Vt(r,i),$p(t,e,r,i,n);case 3:e:{if(qv(e),t===null)throw Error(E(387));r=e.pendingProps,s=e.memoizedState,i=s.element,hv(t,e),ia(e,r,null,n);var o=e.memoizedState;if(r=o.element,s.isDehydrated)if(s={element:r,isDehydrated:!1,cache:o.cache,pendingSuspenseBoundaries:o.pendingSuspenseBoundaries,transitions:o.transitions},e.updateQueue.baseState=s,e.memoizedState=s,e.flags&256){i=Fi(Error(E(423)),e),e=Gp(t,e,r,n,i);break e}else if(r!==i){i=Fi(Error(E(424)),e),e=Gp(t,e,r,n,i);break e}else for(bt=nr(e.stateNode.containerInfo.firstChild),Rt=e,ye=!0,Ut=null,n=cv(e,null,r,n),e.child=n;n;)n.flags=n.flags&-3|4096,n=n.sibling;else{if(_i(),r===i){e=On(t,e,n);break e}nt(t,e,r,n)}e=e.child}return e;case 5:return fv(e),t===null&&Yu(e),r=e.type,i=e.pendingProps,s=t!==null?t.memoizedProps:null,o=i.children,Uu(r,i)?o=null:s!==null&&Uu(r,s)&&(e.flags|=32),_v(t,e),nt(t,e,o,n),e.child;case 6:return t===null&&Yu(e),null;case 13:return Wv(t,e,n);case 4:return Wf(e,e.stateNode.containerInfo),r=e.pendingProps,t===null?e.child=qi(e,null,r,n):nt(t,e,r,n),e.child;case 11:return r=e.type,i=e.pendingProps,i=e.elementType===r?i:Vt(r,i),Hp(t,e,r,i,n);case 7:return nt(t,e,e.pendingProps,n),e.child;case 8:return nt(t,e,e.pendingProps.children,n),e.child;case 12:return nt(t,e,e.pendingProps.children,n),e.child;case 10:e:{if(r=e.type._context,i=e.pendingProps,s=e.memoizedProps,o=i.value,fe(na,r._currentValue),r._currentValue=o,s!==null)if(Xt(s.value,o)){if(s.children===i.children&&!vt.current){e=On(t,e,n);break e}}else for(s=e.child,s!==null&&(s.return=e);s!==null;){var l=s.dependencies;if(l!==null){o=s.child;for(var a=l.firstContext;a!==null;){if(a.context===r){if(s.tag===1){a=An(-1,n&-n),a.tag=2;var c=s.updateQueue;if(c!==null){c=c.shared;var u=c.pending;u===null?a.next=a:(a.next=u.next,u.next=a),c.pending=a}}s.lanes|=n,a=s.alternate,a!==null&&(a.lanes|=n),Xu(s.return,n,e),l.lanes|=n;break}a=a.next}}else if(s.tag===10)o=s.type===e.type?null:s.child;else if(s.tag===18){if(o=s.return,o===null)throw Error(E(341));o.lanes|=n,l=o.alternate,l!==null&&(l.lanes|=n),Xu(o,n,e),o=s.sibling}else o=s.child;if(o!==null)o.return=s;else for(o=s;o!==null;){if(o===e){o=null;break}if(s=o.sibling,s!==null){s.return=o.return,o=s;break}o=o.return}s=o}nt(t,e,i.children,n),e=e.child}return e;case 9:return i=e.type,r=e.pendingProps.children,Ri(e,n),i=Wt(i),r=r(i),e.flags|=1,nt(t,e,r,n),e.child;case 14:return r=e.type,i=Vt(r,e.pendingProps),i=Vt(r.type,i),Up(t,e,r,i,n);case 15:return Nv(t,e,e.type,e.pendingProps,n);case 17:return r=e.type,i=e.pendingProps,i=e.elementType===r?i:Vt(r,i),Al(t,e),e.tag=1,yt(r)?(t=!0,Zl(e)):t=!1,Ri(e,n),Lv(e,r,i),Zu(e,r,i,n),nh(null,e,r,!0,t,n);case 19:return Fv(t,e,n);case 22:return Bv(t,e,n)}throw Error(E(156,e.tag))};function ny(t,e){return A0(t,e)}function Xx(t,e,n,r){this.tag=t,this.key=n,this.sibling=this.child=this.return=this.stateNode=this.type=this.elementType=null,this.index=0,this.ref=null,this.pendingProps=e,this.dependencies=this.memoizedState=this.updateQueue=this.memoizedProps=null,this.mode=r,this.subtreeFlags=this.flags=0,this.deletions=null,this.childLanes=this.lanes=0,this.alternate=null}function Dt(t,e,n,r){return new Xx(t,e,n,r)}function ed(t){return t=t.prototype,!(!t||!t.isReactComponent)}function Jx(t){if(typeof t=="function")return ed(t)?1:0;if(t!=null){if(t=t.$$typeof,t===wf)return 11;if(t===Sf)return 14}return 2}function or(t,e){var n=t.alternate;return n===null?(n=Dt(t.tag,e,t.key,t.mode),n.elementType=t.elementType,n.type=t.type,n.stateNode=t.stateNode,n.alternate=t,t.alternate=n):(n.pendingProps=e,n.type=t.type,n.flags=0,n.subtreeFlags=0,n.deletions=null),n.flags=t.flags&14680064,n.childLanes=t.childLanes,n.lanes=t.lanes,n.child=t.child,n.memoizedProps=t.memoizedProps,n.memoizedState=t.memoizedState,n.updateQueue=t.updateQueue,e=t.dependencies,n.dependencies=e===null?null:{lanes:e.lanes,firstContext:e.firstContext},n.sibling=t.sibling,n.index=t.index,n.ref=t.ref,n}function Ll(t,e,n,r,i,s){var o=2;if(r=t,typeof t=="function")ed(t)&&(o=1);else if(typeof t=="string")o=5;else e:switch(t){case li:return Br(n.children,i,s,e);case yf:o=8,i|=8;break;case Cu:return t=Dt(12,n,e,i|2),t.elementType=Cu,t.lanes=s,t;case bu:return t=Dt(13,n,e,i),t.elementType=bu,t.lanes=s,t;case Tu:return t=Dt(19,n,e,i),t.elementType=Tu,t.lanes=s,t;case f0:return $a(n,i,s,e);default:if(typeof t=="object"&&t!==null)switch(t.$$typeof){case u0:o=10;break e;case h0:o=9;break e;case wf:o=11;break e;case Sf:o=14;break e;case Fn:o=16,r=null;break e}throw Error(E(130,t==null?t:typeof t,""))}return e=Dt(o,n,e,i),e.elementType=t,e.type=r,e.lanes=s,e}function Br(t,e,n,r){return t=Dt(7,t,r,e),t.lanes=n,t}function $a(t,e,n,r){return t=Dt(22,t,r,e),t.elementType=f0,t.lanes=n,t.stateNode={isHidden:!1},t}function jc(t,e,n){return t=Dt(6,t,null,e),t.lanes=n,t}function Vc(t,e,n){return e=Dt(4,t.children!==null?t.children:[],t.key,e),e.lanes=n,e.stateNode={containerInfo:t.containerInfo,pendingChildren:null,implementation:t.implementation},e}function Zx(t,e,n,r,i){this.tag=e,this.containerInfo=t,this.finishedWork=this.pingCache=this.current=this.pendingChildren=null,this.timeoutHandle=-1,this.callbackNode=this.pendingContext=this.context=null,this.callbackPriority=0,this.eventTimes=Cc(0),this.expirationTimes=Cc(-1),this.entangledLanes=this.finishedLanes=this.mutableReadLanes=this.expiredLanes=this.pingedLanes=this.suspendedLanes=this.pendingLanes=0,this.entanglements=Cc(0),this.identifierPrefix=r,this.onRecoverableError=i,this.mutableSourceEagerHydrationData=null}function td(t,e,n,r,i,s,o,l,a){return t=new Zx(t,e,n,l,a),e===1?(e=1,s===!0&&(e|=8)):e=0,s=Dt(3,null,null,e),t.current=s,s.stateNode=t,s.memoizedState={element:r,isDehydrated:n,cache:null,transitions:null,pendingSuspenseBoundaries:null},qf(s),t}function eC(t,e,n){var r=3<arguments.length&&arguments[3]!==void 0?arguments[3]:null;return{$$typeof:oi,key:r==null?null:""+r,children:t,containerInfo:e,implementation:n}}function ry(t){if(!t)return cr;t=t._reactInternals;e:{if(Jr(t)!==t||t.tag!==1)throw Error(E(170));var e=t;do{switch(e.tag){case 3:e=e.stateNode.context;break e;case 1:if(yt(e.type)){e=e.stateNode.__reactInternalMemoizedMergedChildContext;break e}}e=e.return}while(e!==null);throw Error(E(171))}if(t.tag===1){var n=t.type;if(yt(n))return rv(t,n,e)}return e}function iy(t,e,n,r,i,s,o,l,a){return t=td(n,r,!0,t,i,s,o,l,a),t.context=ry(null),n=t.current,r=ot(),i=sr(n),s=An(r,i),s.callback=e??null,rr(n,s,i),t.current.lanes=i,Co(t,i,r),wt(t,r),t}function Ga(t,e,n,r){var i=e.current,s=ot(),o=sr(i);return n=ry(n),e.context===null?e.context=n:e.pendingContext=n,e=An(s,o),e.payload={element:t},r=r===void 0?null:r,r!==null&&(e.callback=r),t=rr(i,e,o),t!==null&&(Yt(t,i,o,s),Rl(t,i,o)),o}function fa(t){if(t=t.current,!t.child)return null;switch(t.child.tag){case 5:return t.child.stateNode;default:return t.child.stateNode}}function rm(t,e){if(t=t.memoizedState,t!==null&&t.dehydrated!==null){var n=t.retryLane;t.retryLane=n!==0&&n<e?n:e}}function nd(t,e){rm(t,e),(t=t.alternate)&&rm(t,e)}function tC(){return null}var sy=typeof reportError=="function"?reportError:function(t){console.error(t)};function rd(t){this._internalRoot=t}Ka.prototype.render=rd.prototype.render=function(t){var e=this._internalRoot;if(e===null)throw Error(E(409));Ga(t,e,null,null)};Ka.prototype.unmount=rd.prototype.unmount=function(){var t=this._internalRoot;if(t!==null){this._internalRoot=null;var e=t.containerInfo;Hr(function(){Ga(null,t,null,null)}),e[Ln]=null}};function Ka(t){this._internalRoot=t}Ka.prototype.unstable_scheduleHydration=function(t){if(t){var e=B0();t={blockedOn:null,target:t,priority:e};for(var n=0;n<Hn.length&&e!==0&&e<Hn[n].priority;n++);Hn.splice(n,0,t),n===0&&q0(t)}};function id(t){return!(!t||t.nodeType!==1&&t.nodeType!==9&&t.nodeType!==11)}function Qa(t){return!(!t||t.nodeType!==1&&t.nodeType!==9&&t.nodeType!==11&&(t.nodeType!==8||t.nodeValue!==" react-mount-point-unstable "))}function im(){}function nC(t,e,n,r,i){if(i){if(typeof r=="function"){var s=r;r=function(){var c=fa(o);s.call(c)}}var o=iy(e,r,t,0,null,!1,!1,"",im);return t._reactRootContainer=o,t[Ln]=o.current,Xs(t.nodeType===8?t.parentNode:t),Hr(),o}for(;i=t.lastChild;)t.removeChild(i);if(typeof r=="function"){var l=r;r=function(){var c=fa(a);l.call(c)}}var a=td(t,0,!1,null,null,!1,!1,"",im);return t._reactRootContainer=a,t[Ln]=a.current,Xs(t.nodeType===8?t.parentNode:t),Hr(function(){Ga(e,a,n,r)}),a}function Ya(t,e,n,r,i){var s=n._reactRootContainer;if(s){var o=s;if(typeof i=="function"){var l=i;i=function(){var a=fa(o);l.call(a)}}Ga(e,o,t,i)}else o=nC(n,e,t,i,r);return fa(o)}O0=function(t){switch(t.tag){case 3:var e=t.stateNode;if(e.current.memoizedState.isDehydrated){var n=ks(e.pendingLanes);n!==0&&(Cf(e,n|1),wt(e,Pe()),!(ne&6)&&(zi=Pe()+500,wr()))}break;case 13:Hr(function(){var r=Dn(t,1);if(r!==null){var i=ot();Yt(r,t,1,i)}}),nd(t,1)}};bf=function(t){if(t.tag===13){var e=Dn(t,134217728);if(e!==null){var n=ot();Yt(e,t,134217728,n)}nd(t,134217728)}};N0=function(t){if(t.tag===13){var e=sr(t),n=Dn(t,e);if(n!==null){var r=ot();Yt(n,t,e,r)}nd(t,e)}};B0=function(){return oe};_0=function(t,e){var n=oe;try{return oe=t,e()}finally{oe=n}};Nu=function(t,e,n){switch(e){case"input":if(Iu(t,n),e=n.name,n.type==="radio"&&e!=null){for(n=t;n.parentNode;)n=n.parentNode;for(n=n.querySelectorAll("input[name="+JSON.stringify(""+e)+'][type="radio"]'),e=0;e<n.length;e++){var r=n[e];if(r!==t&&r.form===t.form){var i=Fa(r);if(!i)throw Error(E(90));p0(r),Iu(r,i)}}}break;case"textarea":g0(t,n);break;case"select":e=n.value,e!=null&&xi(t,!!n.multiple,e,!1)}};C0=Xf;b0=Hr;var rC={usingClientEntryPoint:!1,Events:[To,hi,Fa,k0,x0,Xf]},fs={findFiberByHostInstance:Pr,bundleType:0,version:"18.3.1",rendererPackageName:"react-dom"},iC={bundleType:fs.bundleType,version:fs.version,rendererPackageName:fs.rendererPackageName,rendererConfig:fs.rendererConfig,overrideHookState:null,overrideHookStateDeletePath:null,overrideHookStateRenamePath:null,overrideProps:null,overridePropsDeletePath:null,overridePropsRenamePath:null,setErrorHandler:null,setSuspenseHandler:null,scheduleUpdate:null,currentDispatcherRef:Bn.ReactCurrentDispatcher,findHostInstanceByFiber:function(t){return t=M0(t),t===null?null:t.stateNode},findFiberByHostInstance:fs.findFiberByHostInstance||tC,findHostInstancesForRefresh:null,scheduleRefresh:null,scheduleRoot:null,setRefreshHandler:null,getCurrentFiber:null,reconcilerVersion:"18.3.1-next-f1338f8080-20240426"};if(typeof __REACT_DEVTOOLS_GLOBAL_HOOK__<"u"){var Xo=__REACT_DEVTOOLS_GLOBAL_HOOK__;if(!Xo.isDisabled&&Xo.supportsFiber)try{Ba=Xo.inject(iC),wn=Xo}catch{}}It.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED=rC;It.createPortal=function(t,e){var n=2<arguments.length&&arguments[2]!==void 0?arguments[2]:null;if(!id(e))throw Error(E(200));return eC(t,e,null,n)};It.createRoot=function(t,e){if(!id(t))throw Error(E(299));var n=!1,r="",i=sy;return e!=null&&(e.unstable_strictMode===!0&&(n=!0),e.identifierPrefix!==void 0&&(r=e.identifierPrefix),e.onRecoverableError!==void 0&&(i=e.onRecoverableError)),e=td(t,1,!1,null,null,n,!1,r,i),t[Ln]=e.current,Xs(t.nodeType===8?t.parentNode:t),new rd(e)};It.findDOMNode=function(t){if(t==null)return null;if(t.nodeType===1)return t;var e=t._reactInternals;if(e===void 0)throw typeof t.render=="function"?Error(E(188)):(t=Object.keys(t).join(","),Error(E(268,t)));return t=M0(e),t=t===null?null:t.stateNode,t};It.flushSync=function(t){return Hr(t)};It.hydrate=function(t,e,n){if(!Qa(e))throw Error(E(200));return Ya(null,t,e,!0,n)};It.hydrateRoot=function(t,e,n){if(!id(t))throw Error(E(405));var r=n!=null&&n.hydratedSources||null,i=!1,s="",o=sy;if(n!=null&&(n.unstable_strictMode===!0&&(i=!0),n.identifierPrefix!==void 0&&(s=n.identifierPrefix),n.onRecoverableError!==void 0&&(o=n.onRecoverableError)),e=iy(e,null,t,1,n??null,i,!1,s,o),t[Ln]=e.current,Xs(t),r)for(t=0;t<r.length;t++)n=r[t],i=n._getVersion,i=i(n._source),e.mutableSourceEagerHydrationData==null?e.mutableSourceEagerHydrationData=[n,i]:e.mutableSourceEagerHydrationData.push(n,i);return new Ka(e)};It.render=function(t,e,n){if(!Qa(e))throw Error(E(200));return Ya(null,t,e,!1,n)};It.unmountComponentAtNode=function(t){if(!Qa(t))throw Error(E(40));return t._reactRootContainer?(Hr(function(){Ya(null,null,t,!1,function(){t._reactRootContainer=null,t[Ln]=null})}),!0):!1};It.unstable_batchedUpdates=Xf;It.unstable_renderSubtreeIntoContainer=function(t,e,n,r){if(!Qa(n))throw Error(E(200));if(t==null||t._reactInternals===void 0)throw Error(E(38));return Ya(t,e,n,!1,r)};It.version="18.3.1-next-f1338f8080-20240426";function oy(){if(!(typeof __REACT_DEVTOOLS_GLOBAL_HOOK__>"u"||typeof __REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE!="function"))try{__REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE(oy)}catch(t){console.error(t)}}oy(),o0.exports=It;var sC=o0.exports,ly,sm=sC;ly=sm.createRoot,sm.hydrateRoot;const oC=`// The budget is an ordinary spec. In a spec handler, clocks are those of the
// machine that announced/sent the observed event, so Elapsed(o.started) is
// measured on the client's clock.
spec ResolvesWithinDeadline observes eOutcome {
  start state Watching {
    on eOutcome do (o: (ok: bool, started: int)) {
      assert Elapsed(o.started) <= ms(1000),
        format("resolved (ok = {0}) after up to {1}us; deadline is 1000000us",
               o.ok, Elapsed(o.started));
    }
  }
}
`,lC=`/*****************************************************************************
 * A client calls a flaky server with 3 attempts, a 300ms per-attempt timeout
 * and 50ms / 100ms backoff, and must resolve (success or error) within its
 * caller's 1s deadline. No numbers in this file are delivery latencies; those
 * live in PTst/latency.toml.
 *****************************************************************************/

type tReq  = (client: Client, attempt: int);

event eReq  : tReq;
event eResp : int;          // attempt
event eTimeout : int;       // attempt
event eBackoffDone : int;
event eOutcome : (ok: bool, started: int);   // announced to specs

machine Server {
  start state Serving {
    on eReq do (r: tReq) {
      if ($) { return; }                    // dropped
      Elapse(ms(8), ms(36));
      send r.client, eResp, r.attempt;
    }
  }
}

machine Client {
  var server: Server;
  var attempt: int;
  var started: int;
  var timeoutAfter: int;
  var backoff: seq[int];

  start state Init {
    entry (s: Server) {
      server = s;
      timeoutAfter = ms(300);
      backoff += (0, ms(50));
      backoff += (1, ms(100));
      started = Mark();
      goto Trying;
    }
  }

  state Trying {
    entry {
      attempt = attempt + 1;
      send server, eReq, (client = this, attempt = attempt);
      StartTimer(this, eTimeout, attempt, timeoutAfter);
    }
    on eResp do (a: int) {
      if (a == attempt) { announce eOutcome, (ok = true, started = started); goto Done; }
    }
    on eTimeout do (a: int) {
      if (a != attempt) { return; }
      if (attempt == 3) { announce eOutcome, (ok = false, started = started); goto Done; }
      StartTimer(this, eBackoffDone, attempt, backoff[attempt - 1]);
      goto BackingOff;
    }
  }

  state BackingOff {
    on eBackoffDone goto Trying;
    ignore eResp, eTimeout;
  }

  state Done {
    ignore eResp, eTimeout, eBackoffDone;
  }
}
`,aC=`/*****************************************************************************
 * The entire time "extension": plain P foreign functions. Times are int
 * microseconds. The checker implements them natively (causal clocks, see
 * ../../PROPOSAL.md).
 *****************************************************************************/

// The machine's local clock: worst / best case. Use for absolute deadlines.
fun Now(): int;
fun NowBest(): int;

// A stopwatch. Mark() captures the current clock; Elapsed(m) is the worst
// (ElapsedBest the best) case of now - mark, computed with the correlation
// kept, so shared history cancels. Use these for budgets, not Now() - t0.
fun Mark(): int;
fun Elapsed(m: int): int;
fun ElapsedBest(m: int): int;

// Local work taking between lo and hi microseconds.
fun Elapse(lo: int, hi: int);

fun ms(n: int): int { return n * 1000; }
`,cC=`/*****************************************************************************
 * A one-shot timer, one machine per armed timer.
 *
 * It needs no time semantics of its own: the new machine starts at its
 * creator's clock, Elapse advances that clock by \`after\`, so the timeout it
 * sends is stamped creation + after.
 *   untimed:                   the timeout may be delivered before replies
 *                              stamped earlier, as with today's $-timers.
 *   --timed:                   events certainly arriving earlier are handled first.
 * Cancelling is done by the holder ignoring stale tags, which is simpler and
 * race-free compared to a cancel message.
 *****************************************************************************/

type tTimer = (holder: machine, ev: event, tag: int, after: int);

machine Timer {
  start state Arming {
    entry (t: tTimer) {
      Elapse(t.after, t.after);
      send t.holder, t.ev, t.tag;
    }
  }
}

fun StartTimer(holder: machine, ev: event, tag: int, after: int) {
  new Timer((holder = holder, ev = ev, tag = tag, after = after));
}
`,uC=`machine Main {
  start state Init { entry { new Client(new Server()); } }
}

test tcRetry [main=Main]: assert ResolvesWithinDeadline in { Main, Client, Server, Timer };
`,hC=`# Delivery latencies (\`p check --latency PTst/latency.toml\`).
# Ranges are "lo..hi" in us/ms/s.

[profile.same-az]
default = "0..0"            # anything unlisted, e.g. Timer -> holder
eReq    = "1ms..2ms"
eResp   = "1ms..2ms"

[profile.cross-region]
extends = "same-az"
eReq    = "60ms..90ms"
eResp   = "60ms..90ms"

# Which profile each test case uses unless --latency-profile overrides it.
[tests]
tcRetry = "same-az"
`,fC=`/*****************************************************
This file defines two P specifications

BankBalanceIsAlwaysCorrect (safety property):
BankBalanceIsAlwaysCorrect checks the global invariant that the account-balance communicated
to the client by the bank is always correct and the bank never removes more money from the account
than that withdrawn by the client! Also, if the bank denies a withdraw request then it is only because
the withdrawal would reduce the account balance to below 10.

GuaranteedWithDrawProgress (liveness property):
GuaranteedWithDrawProgress checks the liveness (or progress) property that all withdraw requests
submitted by the client are eventually responded.

Note: stating that "BankBalanceIsAlwaysCorrect checks that if the bank denies a withdraw request
then the request would reduce the balance to below 10 (< 10)" is equivalent to state that "if there is enough money in the account - at least 10 (>= 10), then the request must not error".
Hence, the two properties BankBalanceIsAlwaysCorrect and GuaranteedWithDrawProgress together ensure that every withdraw request if allowed will eventually succeed and the bank cannot block correct withdrawal requests.
*****************************************************/

// event: initialize the monitor with the initial account balances for all clients when the system starts
event eSpec_BankBalanceIsAlwaysCorrect_Init: map[int, int];

/****************************************************
BankBalanceIsAlwaysCorrect checks the global invariant that the account balance communicated
to the client by the bank is always correct and there is no error on the banks side with the
implementation of the withdraw logic.

For checking this property the spec machine observes the withdraw request (eWithDrawReq) and response (eWithDrawResp).
- On receiving the eWithDrawReq, it adds the request in the pending-withdraws-map so that on receiving a
response for this withdraw we can assert that the amount of money deducted from the account is same as
what was requested by the client.

- On receiving the eWithDrawResp, we look up the corresponding withdraw request and check that: the
new account balance is correct and if the withdraw failed it is because the withdraw will make the account
balance go below 10 dollars which is against the bank policies!
****************************************************/
spec BankBalanceIsAlwaysCorrect observes eWithDrawReq,  eWithDrawResp, eSpec_BankBalanceIsAlwaysCorrect_Init {
  // keep track of the bank balance for each client: map from accountId to bank balance.
  var bankBalance: map[int, int];
  // keep track of the pending withdraw requests that have not been responded yet.
  // map from reqId -> withdraw request
  var pendingWithDraws: map[int, tWithDrawReq];

  start state Init {
    on eSpec_BankBalanceIsAlwaysCorrect_Init goto WaitForWithDrawReqAndResp with (balance: map[int, int]){
      bankBalance = balance;
    }
  }

  state WaitForWithDrawReqAndResp {
    on eWithDrawReq do (req: tWithDrawReq) {
      assert req.accountId in bankBalance,
        format ("Unknown accountId {0} in the withdraw request. Valid accountIds = {1}",
          req.accountId, keys(bankBalance));
      pendingWithDraws[req.rId] = req;
    }

    on eWithDrawResp do (resp: tWithDrawResp) {
      assert resp.accountId in bankBalance,
        format ("Unknown accountId {0} in the withdraw response!", resp.accountId);
      assert resp.rId in pendingWithDraws,
        format ("Unknown rId {0} in the withdraw response!", resp.rId);
      assert resp.balance >= 10,
        "Bank balance in all accounts must always be greater than or equal to 10!!";

      if(resp.status == WITHDRAW_SUCCESS)
      {
        assert resp.balance == bankBalance[resp.accountId] - pendingWithDraws[resp.rId].amount,
          format ("Bank balance for the account {0} is {1} and not the expected value {2}, Bank is lying!",
            resp.accountId, resp.balance, bankBalance[resp.accountId] - pendingWithDraws[resp.rId].amount);
        // update the new account balance
        bankBalance[resp.accountId] = resp.balance;
      }
      else
      {
        // bank can only reject a request if it will drop the balance below 10
        assert bankBalance[resp.accountId] - pendingWithDraws[resp.rId].amount < 10,
          format ("Bank must accept the withdraw request for {0}, bank balance is {1}!",
            pendingWithDraws[resp.rId].amount, bankBalance[resp.accountId]);
        // if withdraw failed then the account balance must remain the same
        assert bankBalance[resp.accountId] == resp.balance,
          format ("Withdraw failed BUT the account balance changed! actual: {0}, bank said: {1}",
            bankBalance[resp.accountId], resp.balance);
      }
    }
  }
}

/**************************************************************************
GuaranteedWithDrawProgress checks the liveness (or progress) property that all withdraw requests
submitted by the client are eventually responded.
***************************************************************************/
spec GuaranteedWithDrawProgress observes eWithDrawReq, eWithDrawResp {
  // keep track of the pending withdraw requests
  var pendingWDReqs: set[int];

  start state NopendingRequests {
    on eWithDrawReq goto PendingReqs with (req: tWithDrawReq) {
      pendingWDReqs += (req.rId);
    }
  }

  hot state PendingReqs {
    on eWithDrawResp do (resp: tWithDrawResp) {
      assert resp.rId in pendingWDReqs,
        format ("unexpected rId: {0} received, expected one of {1}", resp.rId, pendingWDReqs);
      pendingWDReqs -= (resp.rId);
      if(sizeof(pendingWDReqs) == 0) // all requests have been responded
        goto NopendingRequests;
    }

    on eWithDrawReq goto PendingReqs with (req: tWithDrawReq){
      pendingWDReqs += (req.rId);
    }
  }
}
`,dC=`/*********************************************************
The AbstractBankServer provides an abstract implementation of the BankServer where it abstract away
the interaction between the BankServer and Database.
The AbstractBankServer machine is used to demonstrate how one can replace a complex component in P
with its abstraction that hides a lot of its internal complexity.
In this case, instead of storing the balance in a separate database the abstraction store the information
locally and abstracts away the complexity of bank server interaction with the database.
For the client, it still exposes the same interface/behavior. Hence, when checking the correctness
of the client it doesnt matter whether we use BankServer or the AbstractBankServer
**********************************************************/

machine AbstractBankServer
{
  // account balance: map from account-id to balance
  var balance: map[int, int];
  start state WaitForWithdrawRequests {
    entry (init_balance: map[int, int])
    {
      balance = init_balance;
    }

    on eWithDrawReq do (wReq: tWithDrawReq) {
      assert wReq.accountId in balance, "Invalid accountId received in the withdraw request!";
      if(balance[wReq.accountId] - wReq.amount > 10) /* hint: bug */
      {
        balance[wReq.accountId] = balance[wReq.accountId] - wReq.amount;
        send wReq.source, eWithDrawResp,
          (status = WITHDRAW_SUCCESS, accountId = wReq.accountId, balance = balance[wReq.accountId], rId = wReq.rId);
      }
      else
      {
        send wReq.source, eWithDrawResp,
          (status = WITHDRAW_ERROR, accountId = wReq.accountId, balance = balance[wReq.accountId], rId = wReq.rId);
      }
    }
  }
}`,pC=`/* User Defined Types */

// payload type associated with the eWithDrawReq, where \`source\`: client sending the withdraw request,
// \`accountId\`: account to withdraw from, \`amount\`: amount to withdraw, and \`rId\`: unique
// request Id associated with each request.
type tWithDrawReq = (source: Client, accountId: int, amount: int, rId:int);

// payload type associated with the eWithDrawResp, where \`status\`: response status (below),
// \`accountId\`: account withdrawn from, \`balance\`: bank balance after withdrawal, and
// \`rId\`: request id for which this is the response.
type tWithDrawResp = (status: tWithDrawRespStatus, accountId: int, balance: int, rId: int);

// enum representing the response status for the withdraw request
enum tWithDrawRespStatus {
  WITHDRAW_SUCCESS,
  WITHDRAW_ERROR
}

// event: withdraw request (from client to bank server)
event eWithDrawReq : tWithDrawReq;
// event: withdraw response (from bank server to client)
event eWithDrawResp: tWithDrawResp;


machine Client
{
  var server : BankServer;
  var accountId: int;
  var nextReqId : int;
  var numOfWithdrawOps: int;
  var currentBalance: int;

  start state Init {

    entry (input : (serv : BankServer, accountId: int, balance : int))
    {
      server = input.serv;
      currentBalance =  input.balance;
      accountId = input.accountId;
      // hacky: we would like request id's to be unique across all requests from clients
      nextReqId = accountId*100 + 1; // each client has a unique account id
      goto WithdrawMoney;
    }
  }

  state WithdrawMoney {
    entry {
      // If current balance is <= 10 then we need more deposits before any more withdrawal
      if(currentBalance <= 10)
        goto NoMoneyToWithDraw;

      // send withdraw request to the bank for a random amount between (1 to current balance + 1)
      send server, eWithDrawReq, (source = this, accountId = accountId, amount = WithdrawAmount(), rId = nextReqId);
      nextReqId = nextReqId + 1;
    }

    on eWithDrawResp do (resp: tWithDrawResp) {
      // bank always ensures that a client has atleast 10 dollars in the account
      assert resp.balance >= 10, "Bank balance must be greater than 10!!";
      if(resp.status == WITHDRAW_SUCCESS) // withdraw succeeded
      {
        print format ("Withdrawal with rId = {0} succeeded, new account balance = {1}", resp.rId, resp.balance);
        currentBalance = resp.balance;
      }
      else // withdraw failed
      {
        // if withdraw failed then the account balance must remain the same
        assert currentBalance == resp.balance,
          format ("Withdraw failed BUT the account balance changed! client thinks: {0}, bank balance: {1}", currentBalance, resp.balance);
        print format ("Withdrawal with rId = {0} failed, account balance = {1}", resp.rId, resp.balance);
      }

      if (currentBalance > 10)
/* Hint 3: Reduce the number of times WithdrawAmount() is called by changing the above line to the following:
      if (currentBalance > 10 && nextReqId < (accountId*100 + 5))
*/
      {
        print format ("Still have account balance = {0}, lets try and withdraw more", currentBalance);
        goto WithdrawMoney;
      }
    }
  }

  // function that returns a random integer between (1 to current balance + 1)
  fun WithdrawAmount() : int {
    return choose(currentBalance) + 1;
/* Hint 2: Reduce the number of choices by changing the above line to the following:
    return ((choose(5) * currentBalance) / 4) + 1;
*/
  }

  state NoMoneyToWithDraw {
    entry {
      // if I am here then the amount of money in my account should be exactly 10
      assert currentBalance == 10, "Hmm, I still have money that I can withdraw but I have reached NoMoneyToWithDraw state!";
      print format ("No Money to withdraw, waiting for more deposits!");
    }
  }
}
`,mC=`// Client module
module Client = { Client };

// Bank module
module Bank = { BankServer, Database };

// Abstract Bank Server module
module AbstractBank = { AbstractBankServer -> BankServer };`,gC=`/** Events used to communicate between the bank server and the backend database **/
// event: send update the database, i.e. the \`balance\` associated with the \`accountId\`
event eUpdateQuery: (accountId: int, balance: int);
// event: send a read request for the \`accountId\`.
event eReadQuery: (accountId: int);
// event: send a response (\`balance\`) corresponding to the read request for an \`accountId\`
event eReadQueryResp: (accountId: int, balance: int);

/*************************************************************
The BankServer machine uses a database machine as a service to store the bank balance for all its clients.
On receiving an eWithDrawReq (withdraw requests) from a client, it reads the current balance for the account,
if there is enough money in the account then it updates the new balance in the database after withdrawal
and sends a response back to the client.
*************************************************************/
machine BankServer
{
  var database: Database;

  start state Init {
    entry (initialBalance: map[int, int]){
      database = new Database((server = this, initialBalance = initialBalance));
      goto WaitForWithdrawRequests;
    }
  }

  state WaitForWithdrawRequests {
    on eWithDrawReq do (wReq: tWithDrawReq) {
      var currentBalance: int;
      var response: tWithDrawResp;

      // read the current account balance from the database
      currentBalance = ReadBankBalance(database, wReq.accountId);
      // if there is enough money in account after withdrawal
      if(currentBalance - wReq.amount >= 10)
      {
        UpdateBankBalance(database, wReq.accountId, currentBalance - wReq.amount);
        response = (status = WITHDRAW_SUCCESS, accountId = wReq.accountId, balance = currentBalance - wReq.amount, rId = wReq.rId);
      }
      else // not enough money after withdraw
      {
        response = (status = WITHDRAW_ERROR, accountId = wReq.accountId, balance = currentBalance, rId = wReq.rId);
      }

      // send response to the client
      send wReq.source, eWithDrawResp, response;
    }
  }
}

/***************************************************************
The Database machine acts as a helper service for the Bank server and stores the bank balance for
each account. There are two API's or functions to interact with the Database:
ReadBankBalance and UpdateBankBalance.
****************************************************************/
machine Database
{
  var server: BankServer;
  var balance: map[int, int];
  start state Init {
    entry(input: (server : BankServer, initialBalance: map[int, int])){
      server = input.server;
      balance = input.initialBalance;
    }
    on eUpdateQuery do (query: (accountId: int, balance: int)) {
      assert query.accountId in balance, "Invalid accountId received in the update query!";
      balance[query.accountId] = query.balance;
    }
    on eReadQuery do (query: (accountId: int))
    {
      assert query.accountId in balance, "Invalid accountId received in the read query!";
      send server, eReadQueryResp, (accountId = query.accountId, balance = balance[query.accountId]);
    }
  }
}

// Function to read the bank balance corresponding to the accountId
fun ReadBankBalance(database: Database, accountId: int) : int {
    var currentBalance: int;
    send database, eReadQuery, (accountId = accountId,);
    receive {
      case eReadQueryResp: (resp: (accountId: int, balance: int)) {
        currentBalance = resp.balance;
      }
    }
    return currentBalance;
}

// Function to update the account balance for the account Id
fun UpdateBankBalance(database: Database, accId: int, bal: int)
{
  send database, eUpdateQuery, (accountId = accId, balance = bal);
}`,vC=`
// Test driver that checks the system with a single Client.
machine TestWithSingleClient
{
  start state Init {
    entry {
      // since client
      SetupClientServerSystem(1);
    }
  }
}

// Test driver that checks the system with multiple Clients.
machine TestWithMultipleClients
{
  start state Init {
    entry {
      // multiple clients between (2, 4)
      SetupClientServerSystem(choose(3) + 2);
    }
  }
}

param nClients: int;

machine TestWithConfig {
  start state Init {
    entry {
      SetupClientServerSystem(nClients);
    }
  }
}

// creates a random map from accountId's to account balance of size \`numAccounts\`
fun CreateRandomInitialAccounts(numAccounts: int) : map[int, int]
{
  var i: int;
  var bankBalance: map[int, int];
  while(i < numAccounts) {
    bankBalance[i] = choose(100) + 10; // min 10 in the account
/* Hint 1: Reduce the number of choices by changing the above line to the following:
    // bankBalance[i] = choose(10) + 10; // min 10 in the account
*/
    i = i + 1;
  }
  return bankBalance;
}

// setup the client server system with one bank server and \`numClients\` clients.
fun SetupClientServerSystem(numClients: int)
{
  var i: int;
  var server: BankServer;
  var accountIds: seq[int];
  var initAccBalance: map[int, int];

  // randomly initialize the account balance for all clients
  initAccBalance = CreateRandomInitialAccounts(numClients);
  // create bank server with the init account balance
  server = new BankServer(initAccBalance);

  // before client starts sending any messages make sure we
  // initialize the monitors or specifications
  announce eSpec_BankBalanceIsAlwaysCorrect_Init, initAccBalance;

  accountIds = keys(initAccBalance);

  // create the clients
  while(i < sizeof(accountIds)) {
    new Client((serv = server, accountId = accountIds[i], balance = initAccBalance[accountIds[i]]));
    i = i + 1;
  }
}`,yC=`/* This file contains three different model checking scenarios */

// assert the properties for the single client and single server scenario
test tcSingleClient [main=TestWithSingleClient]:
  assert BankBalanceIsAlwaysCorrect, GuaranteedWithDrawProgress in
  (union Client, Bank, { TestWithSingleClient });

// assert the properties for the two clients and single server scenario
test tcMultipleClients [main=TestWithMultipleClients]:
  assert BankBalanceIsAlwaysCorrect, GuaranteedWithDrawProgress in
  (union Client, Bank, { TestWithMultipleClients });

// assert the properties for the single client and single server scenario but with abstract server
 test tcAbstractServer [main=TestWithSingleClient]:
  assert BankBalanceIsAlwaysCorrect, GuaranteedWithDrawProgress in
  (union Client, AbstractBank, { TestWithSingleClient });

// test with a parameterized number of clients
test param (nClients in [2, 3, 4]) tcParameterizedMultipleClients [main=TestWithConfig]:
  assert BankBalanceIsAlwaysCorrect, GuaranteedWithDrawProgress in
  (union Client, Bank, { TestWithConfig });
`,wC=`// event: initialize the AtomicityInvariant spec monitor
event eMonitor_AtomicityInitialize: int;

/**********************************
We would like to assert the atomicity property that:
if a transaction is committed by the coordinator then it was agreed on by all participants
***********************************/
spec AtomicityInvariant observes eWriteTransResp, ePrepareResp, eMonitor_AtomicityInitialize
{
  // a map from transaction id to a map from responses status to number of participants with that response
  var participantsResponse: map[int, map[tTransStatus, int]];
  var numParticipants: int;
  start state Init {
    on eMonitor_AtomicityInitialize goto WaitForEvents with (n: int) {
      numParticipants = n;
    }
  }

  state WaitForEvents {
    on ePrepareResp do (resp: tPrepareResp){
      var transId: int;
      transId = resp.transId;

      if(!(transId in participantsResponse))
      {
        participantsResponse[transId] = default(map[tTransStatus, int]);
        participantsResponse[transId][SUCCESS] = 0;
        participantsResponse[transId][ERROR] = 0;
      }
      participantsResponse[transId][resp.status] = participantsResponse[transId][resp.status] + 1;
    }

    on eWriteTransResp do (resp: tWriteTransResp) {
      assert (resp.transId in participantsResponse || resp.status == TIMEOUT),
      format ("Write transaction was responded to the client without receiving any responses from the participants!");

      if(resp.status == SUCCESS)
      {
        assert participantsResponse[resp.transId][SUCCESS] == numParticipants,
        format ("Write transaction was responded as committed before receiving success from all participants. ") +
        format ("participants sent success: {0}, participants sent error: {1}", participantsResponse[resp.transId][SUCCESS],
        participantsResponse[resp.transId][ERROR]);
      }
      else if(resp.status == ERROR)
      {
        assert participantsResponse[resp.transId][ERROR] > 0,
          format ("Write transaction {0} was responded as failed before receiving error from atleast one participant.", resp.transId) +
          format ("participants sent success: {0}, participants sent error: {1}", participantsResponse[resp.transId][SUCCESS],
            participantsResponse[resp.transId][ERROR]);
      }
      // remove the transaction information
      participantsResponse -= (resp.transId);
    }
  }
}

/**************************************************************************
Every received transaction from a client must be eventually responded back.
Note, the usage of hot and cold states.
***************************************************************************/
spec Progress observes eWriteTransReq, eWriteTransResp {
  var pendingTransactions: int;
  start state Init {
    on eWriteTransReq goto WaitForResponses with { pendingTransactions = pendingTransactions + 1; }
  }

  hot state WaitForResponses
  {
    on eWriteTransResp do {
      pendingTransactions = pendingTransactions - 1;
      if(pendingTransactions == 0)
      {
        goto AllTransactionsFinished;
      }
    }

    on eWriteTransReq do { pendingTransactions = pendingTransactions + 1; }
  }

  cold state AllTransactionsFinished {
    on eWriteTransReq goto WaitForResponses with { pendingTransactions = pendingTransactions + 1; }
  }
}
`,SC=`/* User Defined Types */

// a transaction consisting of the key, value, and the unique transaction id.
type tTrans = (key: string, val: int, transId: int);
// payload type associated with the \`eWriteTransReq\` event where \`client\`: client sending the
// transaction, \`trans\`: transaction to be committed.
type tWriteTransReq = (client: Client, trans: tTrans);
// payload type associated with the \`eWriteTransResp\` event where \`transId\` is the transaction Id
// and \`status\` is the return status of the transaction request.
type tWriteTransResp = (transId: int, status: tTransStatus);
// payload type associated with the \`eReadTransReq\` event where \`client\` is the Client machine sending
// the read request and \`key\` is the key whose value the client wants to read.
type tReadTransReq = (client: Client, key: string);
// payload type associated with the \`eReadTransResp\` event where \`val\` is the value corresponding to
// the \`key\` in the read request and \`status\` is the read status (e.g., success or failure)
type tReadTransResp = (key: string, val: int, status: tTransStatus);

// transaction status
enum tTransStatus {
  SUCCESS,
  ERROR,
  TIMEOUT
}

/* Events used by the 2PC clients to communicate with the 2PC coordinator */
// event: write transaction request (client to coordinator)
event eWriteTransReq : tWriteTransReq;
// event: write transaction response (coordinator to client)
event eWriteTransResp : tWriteTransResp;
// event: read transaction request (client to coordinator)
event eReadTransReq : tReadTransReq;
// event: read transaction response (participant to client)
event eReadTransResp: tReadTransResp;

/* Events used for communication between the coordinator and the participants */
// event: prepare request for a transaction (coordinator to participant)
event ePrepareReq: tPrepareReq;
// event: prepare response for a transaction (participant to coodinator)
event ePrepareResp: tPrepareResp;
// event: commit transaction (coordinator to participant)
event eCommitTrans: int;
// event: abort transaction (coordinator to participant)
event eAbortTrans: int;

/* User Defined Types */
// payload type associated with the \`ePrepareReq\` event
type tPrepareReq = tTrans;
// payload type assocated with the \`ePrepareResp\` event where \`participant\` is the participant machine
// sending the response, \`transId\` is the transaction id, and \`status\` is the status of the prepare
// request for that transaction.
type tPrepareResp = (participant: Participant, transId: int, status: tTransStatus);

// event: inform participant about the coordinator
event eInformCoordinator: Coordinator;

/*****************************************************************************************
The Coordinator machine receives write and read transactions from the clients. The coordinator machine
services these transactions one by one in the order in which they were received. On receiving a write
transaction the coordinator sends prepare request to all the participants and waits for prepare
responses from all the participants. Based on the responses, the coordinator either commits or aborts
the transaction. If the coordinator fails to receive agreement from participants in time, then it
timesout and aborts the transaction. On receiving a read transaction, the coordinator randomly selects
a participant and  forwards the read request to that participant.
******************************************************************************************/
machine Coordinator
{
  // set of participants
  var participants: set[Participant];
  // current write transaction being handled
  var currentWriteTransReq: tWriteTransReq;
  // previously seen transaction ids
  var seenTransIds: set[int];
  var timer: Timer;

  start state Init {
    entry (payload: set[Participant]){
      participants = payload;
      timer = CreateTimer(this);
      // inform all participants that I am the coordinator
      BroadcastToAllParticipants(eInformCoordinator, this);
      goto WaitForTransactions;
    }
  }

  state WaitForTransactions {
    on eWriteTransReq do (wTrans : tWriteTransReq) {
      if(wTrans.trans.transId in seenTransIds) // transId have to be unique
      {
        send wTrans.client, eWriteTransResp, (transId = wTrans.trans.transId, status = TIMEOUT);
        return;
      }

      currentWriteTransReq = wTrans;
      BroadcastToAllParticipants(ePrepareReq, wTrans.trans);
      //start timer while waiting for responses from all participants
      StartTimer(timer);
      goto WaitForPrepareResponses;
    }

    on eReadTransReq do (rTrans : tReadTransReq) {
      // non-deterministically pick a participant to read from.
      send choose(participants), eReadTransReq, rTrans;
    }

    // when in this state it is fine to drop these messages as they are from the previous transaction
    ignore ePrepareResp, eTimeOut;
  }

  var countPrepareResponses: int;

  state WaitForPrepareResponses {
    // defer requests, we are going to process transactions sequentially
    defer eWriteTransReq;

    on ePrepareResp do (resp : tPrepareResp) {
      // check if the response is for the current transaction else ignore it
      if (currentWriteTransReq.trans.transId == resp.transId) {
        if(resp.status == SUCCESS)
        {
          countPrepareResponses = countPrepareResponses + 1;
          // check if we have received all responses
          if(countPrepareResponses == sizeof(participants))
          {
            // lets commit the transaction
            DoGlobalCommit();
            // safe to go back and service the next transaction
            goto WaitForTransactions;
          }
        }
        else
        {
          DoGlobalAbort(ERROR);
          // safe to go back and service the next transaction
          goto WaitForTransactions;
        }
      }
    }

    // on timeout abort the transaction
    on eTimeOut goto WaitForTransactions with { DoGlobalAbort(TIMEOUT); }

    on eReadTransReq do (rTrans : tReadTransReq) {
      // non-deterministically pick a participant to read from.
      send choose(participants), eReadTransReq, rTrans;
    }

    exit {
      countPrepareResponses = 0;
    }
  }

  fun DoGlobalAbort(respStatus: tTransStatus) {
    // ask all participants to abort and fail the transaction
    BroadcastToAllParticipants(eAbortTrans, currentWriteTransReq.trans.transId);
    send currentWriteTransReq.client, eWriteTransResp, (transId = currentWriteTransReq.trans.transId, status = respStatus);
    if(respStatus != TIMEOUT)
      CancelTimer(timer);
  }

  fun DoGlobalCommit() {
    // ask all participants to commit and respond to client
    BroadcastToAllParticipants(eCommitTrans, currentWriteTransReq.trans.transId);
    send currentWriteTransReq.client, eWriteTransResp,
      (transId = currentWriteTransReq.trans.transId, status = SUCCESS);
    CancelTimer(timer);
  }

  //function to broadcast messages to all participants
  fun BroadcastToAllParticipants(message: event, payload: any)
  {
    var i: int;
    while (i < sizeof(participants)) {
      send participants[i], message, payload;
      i = i + 1;
    }
  }
}

`,kC=`/*****************************************************************************************
Each participant maintains a local key-value store which is updated based on the
transactions committed by the coordinator. On receiving a prepare request
from the coordinator, the participant chooses to either accept or
reject the transaction.
******************************************************************************************/

machine Participant {
  // local key value store
  var kvStore: map[string, tTrans];
  // pending write transactions that have not been committed or aborted yet.
  var pendingWriteTrans: map[int, tTrans];
  // coordinator machine
  var coordinator: Coordinator;

  start state Init {
    on eInformCoordinator goto WaitForRequests with (coor: Coordinator) {
      coordinator = coor;
    }
    defer eShutDown;
  }

  state WaitForRequests {
    on eAbortTrans do (transId: int) {
      // check that abort transaction request is received for a pending transaction only.
      assert transId in pendingWriteTrans,
      format ("Abort request for a non-pending transaction, transId: {0}, pendingTrans set: {1}",
        transId, pendingWriteTrans);
      // remove the transaction from the pending transactions set
      pendingWriteTrans -= (transId);
    }

    on eCommitTrans do (transId:int) {
      // check that  commit transaction request is received for a pending transactions only.
      assert transId in pendingWriteTrans,
      format ("Commit request for a non-pending transaction, transId: {0}, pendingTrans set: {1}",
        transId, pendingWriteTrans);
      // commit the transaction locally
      kvStore[pendingWriteTrans[transId].key] = pendingWriteTrans[transId];
      // remove the transaction from the pending transactions set
      pendingWriteTrans -= (transId);
    }

    on ePrepareReq do (prepareReq :tPrepareReq) {
      // cannot receive prepare for an already pending transaction
      assert !(prepareReq.transId in pendingWriteTrans),
      format ("Duplicate transaction ids not allowed!, received transId: {0}, pending transactions: {1}",
        prepareReq.transId, pendingWriteTrans);
      // add the transaction to the pending transactions set
      pendingWriteTrans[prepareReq.transId] = prepareReq;
      // non-deterministically pick whether to accept or reject the transaction
      if (!(prepareReq.key in kvStore) || (prepareReq.key in kvStore && prepareReq.transId > kvStore[prepareReq.key].transId)) {
        send coordinator, ePrepareResp, (participant = this, transId = prepareReq.transId, status = SUCCESS);
      } else {
        send coordinator, ePrepareResp, (participant = this, transId = prepareReq.transId, status = ERROR);
      }
    }

    on eReadTransReq do (req: tReadTransReq) {
      if(req.key in kvStore)
      {
        // read successful as the key exists
        send req.client, eReadTransResp, (key = req.key, val = kvStore[req.key].val, status = SUCCESS);
      }
      else
      {
        // read failed as the key does not exist
        send req.client, eReadTransResp, (key = "", val = -1, status = ERROR);
      }
    }

    on eShutDown do {
      raise halt;
    }
  }
}

`,xC=`// the two phase commit module
module TwoPhaseCommit = union { Coordinator, Participant }, Timer;`,CC=`/*****************************************************************************************
The client machine below implements the client of the two-phase-commit transaction service.
Each client issues N non-deterministic write-transactions,
if the transaction succeeds then it performs a read-transaction on the same key and asserts the value.
******************************************************************************************/
machine Client {
  // the coordinator machine
  var coordinator: Coordinator;
  // current transaction issued by the client
  var currTransaction : tTrans;
  // number of transactions to be issued
  var N: int;
  // uniqie client Id
  var id: int;

  start state Init {
    entry (payload : (coordinator: Coordinator, n : int, id: int)) {
      coordinator = payload.coordinator;
      N = payload.n;
      id = payload.id;
      goto SendWriteTransaction;
    }
  }

  state SendWriteTransaction {
    entry {
      currTransaction = ChooseRandomTransaction(id * 100 + N /* hack for creating unique transaction id*/);
      send coordinator, eWriteTransReq, (client = this, trans = currTransaction);
    }
    on eWriteTransResp goto ConfirmTransaction;
  }

  state ConfirmTransaction {
    entry (writeResp: tWriteTransResp) {
      // assert that if write transaction was successful then value read is the value written.
      if(writeResp.status == SUCCESS)
      {
        send coordinator, eReadTransReq, (client= this, key = currTransaction.key);
        // await response from the participant
        receive {
          case eReadTransResp: (readResp: tReadTransResp) {
            assert readResp.key == currTransaction.key && readResp.val == currTransaction.val,
              format ("Record read is not same as what was written by the client:: read - {0}, written - {1}",
            readResp.val, currTransaction.val);
          }
        }
      }
      // has more work to do?
      if(N > 0)
      {
        N = N - 1;
        goto SendWriteTransaction;
      }
    }
  }
}

fun ChooseRandomTransaction(uniqueId: int): tTrans {
  return (key = format("{0}", choose(10)), val = choose(10), transId = uniqueId);
}


// two phase commit client module
module TwoPCClient = { Client };
`,bC=`// type that represents the configuration of the system under test
type t2PCConfig = (
  numClients: int,
  numParticipants: int,
  numTransPerClient: int,
  failParticipants: int
);

// function that creates the two phase commit system along with the machines in its
// environment (test harness)
fun SetUpTwoPhaseCommitSystem(config: t2PCConfig)
{
  var coordinator : Coordinator;
  var participants: set[Participant];
  var i : int;

  // create participants
  while (i < config.numParticipants) {
    participants += (new Participant());
    i = i + 1;
  }

  // initialize the monitors (specifications)
  InitializeTwoPhaseCommitSpecifications(config.numParticipants);

  // create the coordinator
  coordinator = new Coordinator(participants);

  // create the clients
  i = 0;
  while(i < config.numClients)
  {
    new Client((coordinator = coordinator, n = config.numTransPerClient, id = i + 1));
    i = i + 1;
  }

  // create the failure injector if we want to inject failures
  if(config.failParticipants > 0)
  {
    CreateFailureInjector((nodes = participants, nFailures = config.failParticipants));
  }
}

fun InitializeTwoPhaseCommitSpecifications(numParticipants: int) {
  // inform the monitor the number of participants in the system
  announce eMonitor_AtomicityInitialize, numParticipants;
}

/*
This machine creates the 3 participants, 1 coordinator, and 1 clients
*/
machine SingleClientNoFailure {
  start state Init {
    entry {
      var config: t2PCConfig;

      config = (numClients = 1,
                      numParticipants = 3,
                      numTransPerClient = 2,
                      failParticipants = 0);

            SetUpTwoPhaseCommitSystem(config);
    }
  }
}

/*
This machine creates the 3 participants, 1 coordinator, and 2 clients
*/
machine MultipleClientsNoFailure {
  start state Init {
    entry {
      var config: t2PCConfig;
      config =
        (numClients = 2,
        numParticipants = 3,
        numTransPerClient = 2,
        failParticipants = 0);

        SetUpTwoPhaseCommitSystem(config);
    }
  }
}

/*
This machine creates the 3 participants, 1 coordinator, 1 Failure injector, and 2 clients
*/
machine MultipleClientsWithFailure {
  start state Init {
    entry {
      var config: t2PCConfig;
      config =
        (numClients = 2,
        numParticipants = 3,
        numTransPerClient = 2,
        failParticipants = 1);

      SetUpTwoPhaseCommitSystem(config);
    }
  }
}

// Parameters for system configuration
param pNumClients: int;
param pNumParticipants: int;
param pNumTransPerClient: int;
param pFailParticipants: int;
/*
This machine allows parameterized testing with configurable number of clients, participants, and failures
*/

machine TestWithConfig {
  start state Init {
    entry {
      var config: t2PCConfig;
      config = (
        numClients = pNumClients,
        numParticipants = pNumParticipants,
        numTransPerClient = pNumTransPerClient,
        failParticipants = pFailParticipants
      );

      SetUpTwoPhaseCommitSystem(config);
    }
  }
}
`,TC=`// checks that all events are handled correctly and also the local assertions in the P machines.
test tcSingleClientNoFailure [main = SingleClientNoFailure]:
  union TwoPhaseCommit, TwoPCClient, FailureInjector, { SingleClientNoFailure };

// asserts the liveness monitor along with the default properties
test tcMultipleClientsNoFailure [main = MultipleClientsNoFailure]:
  assert AtomicityInvariant, Progress in
    (union TwoPhaseCommit, TwoPCClient, FailureInjector, { MultipleClientsNoFailure });

// asserts the liveness monitor along with the default properties
test tcMultipleClientsWithFailure [main = MultipleClientsWithFailure]:
  assert Progress in (union TwoPhaseCommit, TwoPCClient, FailureInjector, { MultipleClientsWithFailure });

// parametric testing of all parameters
test param (pNumClients in [2, 3], pNumParticipants in [3, 4, 5], 
           pNumTransPerClient in [1, 2], pFailParticipants in [0, 1])
  assume (pNumParticipants > pNumClients && pFailParticipants < pNumParticipants/2)
  (2 wise) tcParametricTests [main=TestWithConfig]:
  assert AtomicityInvariant, Progress in
  (union TwoPhaseCommit, TwoPCClient, FailureInjector, { TestWithConfig });
`,RC=`/* Events used to inform monitor about the internal state of the CoffeeMaker */
event eInWarmUpState;
event eInReadyState;
event eInBeansGrindingState;
event eInCoffeeBrewingState;
event eErrorHappened;
event eResetPerformed;

/*********************************************
We would like to ensure that the coffee maker moves
through the expected modes of operation. We want to make sure that the coffee maker always transitions
through the following sequence of states:
Steady operation:
  WarmUp -> Ready -> GrindBeans -> MakeCoffee -> Ready
With Error:
If an error occurs in any of the states above then the Coffee machine stays in the error state until
it is reset and after which it returns to the Warmup state.

**********************************************/
spec EspressoMachineModesOfOperation
observes eInWarmUpState, eInReadyState, eInBeansGrindingState, eInCoffeeBrewingState, eErrorHappened, eResetPerformed
{
  start state StartUp {
    on eInWarmUpState goto WarmUp;
  }

  state WarmUp {
    on eErrorHappened goto Error;
    on eInReadyState goto Ready;
  }

  state Ready {
    ignore eInReadyState;
    on eInBeansGrindingState goto BeanGrinding;
    on eErrorHappened goto Error;
  }

  state BeanGrinding {
    on eInCoffeeBrewingState goto MakingCoffee;
    on eErrorHappened goto Error;
  }

  state MakingCoffee {
    on eInReadyState goto Ready;
    on eErrorHappened goto Error;
  }

  state Error {
    on eResetPerformed goto StartUp;
    ignore eErrorHappened;
  }
}

`,MC=`
/* Requests or operations from the controller to coffee maker */

// event: warmup request when the coffee maker starts or resets
event eWarmUpReq;
// event: grind beans request before making coffee
event eGrindBeansReq;
// event: start brewing coffee
event eStartEspressoReq;
// event start steamer
event eStartSteamerReq;
// event: stop steamer
event eStopSteamerReq;

/* Responses from the coffee maker to the controller */
// event: completed grinding beans
event eGrindBeansCompleted;
// event: completed brewing and pouring coffee
event eEspressoCompleted;
// event: warmed up the machine and read to make coffee
event eWarmUpCompleted;

/* Error messages from the coffee maker to control panel or controller*/
// event: no water for coffee, refill water!
event eNoWaterError;
// event: no beans for coffee, refill beans!
event eNoBeansError;
// event: the heater to warm the machine is broken!
event eWarmerError;

/*****************************************************
EspressoCoffeeMaker receives requests from the control panel of the coffee machine and
based on its state e.g., whether heater is working, or it has beans and water, the maker responds
back to the controller if the operation succeeded or errored.
*****************************************************/
machine EspressoCoffeeMaker
{
  // control panel of the coffee machine that sends inputs to the coffee maker
  var controller: CoffeeMakerControlPanel;

  start state WaitForRequests {
    entry (_controller: CoffeeMakerControlPanel) {
      controller = _controller;
    }

    on eWarmUpReq do {
        send controller, eWarmUpCompleted;
    }

    on eGrindBeansReq do {
      if (!HasBeans()) {
        send controller, eNoBeansError;
      } else {
        send controller, eGrindBeansCompleted;
      }
    }

    on eStartEspressoReq do {
      if (!HasWater()) {
        send controller, eNoWaterError;
      } else {
        send controller, eEspressoCompleted;
      }
    }
    on eStartSteamerReq do {
      if (!HasWater()) {
        send controller, eNoWaterError;
      }
    }
    on eStopSteamerReq do { /* do nothing, steamer stopped */ }
  }

  // nondeterministic functions to trigger different behaviors
  fun HasBeans() : bool { return $; }
  fun HasWater() : bool { return $; }
}


`,IC=`/* Events used by the user to interact with the control panel of the Coffee Machine */
// event: make espresso button pressed
event eEspressoButtonPressed;
// event: steamer button turned off
event eSteamerButtonOff;
// event: steamer button turned on
event eSteamerButtonOn;
// event: door opened to empty grounds
event eOpenGroundsDoor;
// event: door closed after emptying grounds
event eCloseGroundsDoor;
// event: reset coffee maker button pressed
event eResetCoffeeMaker;
//event: error message from panel to the user
event eCoffeeMakerError: tCoffeeMakerState;
//event: coffee machine is ready
event eCoffeeMakerReady;
// event: coffee machine user
event eCoffeeMachineUser: machine;

// enum to represent the state of the coffee maker
enum tCoffeeMakerState {
  NotWarmedUp,
  Ready,
  NoBeansError,
  NoWaterError
}

/*
CoffeeMakerControlPanel acts as the interface between the CoffeeMaker and User
It converts the inputs from the user to appropriate inputs to the CoffeeMaker and sends responses to
the user.
*/
machine CoffeeMakerControlPanel
{
  var coffeeMaker: EspressoCoffeeMaker;
  var coffeeMakerState: tCoffeeMakerState;
  var currentUser: machine;

  start state Init {
    entry {
      coffeeMakerState = NotWarmedUp;
      coffeeMaker = new EspressoCoffeeMaker(this);
      WaitForUser();
      goto WarmUpCoffeeMaker;
    }
  }

  // block until a user shows up
  fun WaitForUser() {
      receive {
          case eCoffeeMachineUser: (user: machine) {
              currentUser = user;
          }
      }
  }

  state WarmUpCoffeeMaker {
    entry {
      // inform the specification about current state of the coffee maker
      announce eInWarmUpState;

      BeginHeatingCoffeeMaker();
    }

    on eWarmUpCompleted goto CoffeeMakerReady;

    // grounds door is opened or closed will handle it later after the coffee maker has warmed up
    defer eOpenGroundsDoor, eCloseGroundsDoor;
    // ignore these inputs from users until the maker has warmed up.
    ignore eEspressoButtonPressed, eSteamerButtonOff, eSteamerButtonOn, eResetCoffeeMaker;
    // ignore these errors and responses as they could be from previous state
    ignore eNoBeansError, eNoWaterError, eGrindBeansCompleted;
  }




  state CoffeeMakerReady {
    entry {
      // inform the specification about current state of the coffee maker
      announce eInReadyState;

      coffeeMakerState = Ready;
      send currentUser, eCoffeeMakerReady;
    }

    on eOpenGroundsDoor goto CoffeeMakerDoorOpened;
    on eEspressoButtonPressed goto CoffeeMakerRunGrind;
    on eSteamerButtonOn goto CoffeeMakerRunSteam;

    // ignore these out of order commands, these must have happened because of an error
    // from user or sensor
    ignore eSteamerButtonOff, eCloseGroundsDoor;

    // ignore commands and errors as they are from previous state
    ignore eWarmUpCompleted, eResetCoffeeMaker, eNoBeansError, eNoWaterError;
  }

  state CoffeeMakerRunGrind {
    entry {
      // inform the specification about current state of the coffee maker
      announce eInBeansGrindingState;

      GrindBeans();
    }
    on eNoBeansError goto EncounteredError with {
      coffeeMakerState = NoBeansError;
      print "No beans to grind! Please refill beans and reset the machine!";
    }

    on eNoWaterError goto EncounteredError with {
      coffeeMakerState = NoWaterError;
      print "No Water! Please refill water and reset the machine!";
    }

    on eGrindBeansCompleted goto CoffeeMakerRunEspresso;

    defer eOpenGroundsDoor, eCloseGroundsDoor, eEspressoButtonPressed;

    // Can't make steam while we are making espresso
    ignore eSteamerButtonOn, eSteamerButtonOff;

    // ignore commands that are old or cannot be handled right now
    ignore eWarmUpCompleted, eResetCoffeeMaker;
  }

  state CoffeeMakerRunEspresso {
    entry {
      // inform the specification about current state of the coffee maker
      announce eInCoffeeBrewingState;

      StartEspresso();
    }
    on eEspressoCompleted goto CoffeeMakerReady with { send currentUser, eEspressoCompleted; }

    on eNoWaterError goto EncounteredError with {
      coffeeMakerState = NoWaterError;
      print "No Water! Please refill water and reset the machine!";
    }

    // the user commands will be handled next after finishing this espresso
    defer eOpenGroundsDoor, eCloseGroundsDoor, eEspressoButtonPressed;

    // Can't make steam while we are making espresso
    ignore eSteamerButtonOn, eSteamerButtonOff;

    // ignore old commands and cannot reset when making coffee
    ignore eWarmUpCompleted, eResetCoffeeMaker;
  }

  state CoffeeMakerRunSteam {
    entry {
      StartSteamer();
    }

    on eSteamerButtonOff  goto CoffeeMakerReady with {
      StopSteamer();
    }

    on eNoWaterError goto EncounteredError with {
      coffeeMakerState = NoWaterError;
      print "No Water! Please refill water and reset the machine!";
    }

    // user might have cleaned grounds while steaming
    defer eOpenGroundsDoor, eCloseGroundsDoor;

    // can't make espresso while we are making steam
    ignore eEspressoButtonPressed, eSteamerButtonOn;
  }

  state CoffeeMakerDoorOpened {
    on eCloseGroundsDoor do {
      assert coffeeMakerState != NotWarmedUp;
      assert coffeeMakerState == Ready;
      goto CoffeeMakerReady;
    }

    // grounds door is open cannot handle these requests just ignore them
    ignore eEspressoButtonPressed, eSteamerButtonOn, eSteamerButtonOff;
  }

  state EncounteredError {
    entry {
      // inform the specification about current state of the coffee maker
      announce eErrorHappened;

      // send the error message to the client
      send currentUser, eCoffeeMakerError, coffeeMakerState;
    }

    on eResetCoffeeMaker goto WarmUpCoffeeMaker with {
      // inform the specification about current state of the coffee maker
      announce eResetPerformed;
    }

    // error, ignore these requests until reset.
    ignore eEspressoButtonPressed, eSteamerButtonOn, eSteamerButtonOff,
        eOpenGroundsDoor, eCloseGroundsDoor, eWarmUpCompleted, eEspressoCompleted, eGrindBeansCompleted;

    // ignore other simultaneous errors
    ignore eNoBeansError, eNoWaterError;
  }

  fun BeginHeatingCoffeeMaker() {
    // send an event to maker to start warming
    send coffeeMaker, eWarmUpReq;
  }

  fun StartSteamer() {
    // send an event to maker to start steaming
    send coffeeMaker, eStartSteamerReq;
  }

  fun StopSteamer() {
    // send an event to maker to stop steaming
    send coffeeMaker, eStopSteamerReq;
  }

  fun GrindBeans() {
    // send an event to maker to grind beans
    send coffeeMaker, eGrindBeansReq;
  }

  fun StartEspresso() {
    // send an event to maker to start espresso
    send coffeeMaker, eStartEspressoReq;
  }
}
`,AC="module EspressoMachine = { CoffeeMakerControlPanel, EspressoCoffeeMaker };",PC=`machine TestWithSaneUser
{
  start state Init {
    entry {
      // create a sane user
      new SaneUser(new CoffeeMakerControlPanel());
    }
  }
}

machine TestWithCrazyUser
{
  start state Init {
    entry {
      // create a crazy user
      new CrazyUser((coffeeMaker = new CoffeeMakerControlPanel(), nOps = 5));
    }
  }
}

// Parameters for comprehensive testing scenarios
param nOps : int;
param nUsers : int;
param waterLevel : int;
param beanLevel : int;
param enableSteamer : bool;
param cleaningMode : bool;

machine TestWithConfig
{
  start state Init {
    entry {
      // create a crazy user
      new CrazyUser((coffeeMaker = new CoffeeMakerControlPanel(), nOps = nOps));
    }
  }
}

// Test driver for multiple users, each with their own coffee machine
machine TestWithMultipleUsers
{
  var i: int;
  start state Init {
    entry {
      var coffeeMaker: CoffeeMakerControlPanel;
      
      i = 0;
      while (i < nUsers) {
        coffeeMaker = new CoffeeMakerControlPanel();
        
        if (i % 2 == 0) {
          new SaneUser(coffeeMaker);
        } else {
          new CrazyUser((coffeeMaker = coffeeMaker, nOps = nOps));
        }
        i = i + 1;
      }
    }
  }
}

// Test driver with resource constraints (water and bean levels)
machine TestWithResourceConstraints
{
  start state Init {
    entry {
      var coffeeMaker: CoffeeMakerControlPanel;
      coffeeMaker = new CoffeeMakerControlPanel();
      
      if (waterLevel > 0 && beanLevel > 0) {
        new SaneUser(coffeeMaker);
      } else {
        new CrazyUser((coffeeMaker = coffeeMaker, nOps = 2));
      }
    }
  }
}

// Test driver with mixed configurations
machine TestWithMixedConfiguration
{
  start state Init {
    entry {
      var coffeeMaker: CoffeeMakerControlPanel;
      coffeeMaker = new CoffeeMakerControlPanel();
      
      if (cleaningMode) {
        new CrazyUser((coffeeMaker = coffeeMaker, nOps = 3));
      } else if (enableSteamer) {
        new SteamerTestUser(coffeeMaker);
      } else {
        new SaneUser(coffeeMaker);
      }
    }
  }
}

`,EC=`test tcSaneUserUsingCoffeeMachine [main=TestWithSaneUser]:
  assert EspressoMachineModesOfOperation in (union { TestWithSaneUser }, EspressoMachine, Users);

test tcCrazyUserUsingCoffeeMachine [main=TestWithCrazyUser]:
  assert EspressoMachineModesOfOperation in (union { TestWithCrazyUser }, EspressoMachine, Users);

// BASIC PARAMETER TESTS

// test with a parameterized number of operations for the crazy user
test param (nOps in [4, 5, 6]) tcCrazyUserParamOps [main=TestWithConfig]:
  assert EspressoMachineModesOfOperation in (union EspressoMachine, Users, { TestWithConfig });

// MULTI-PARAMETER TESTS

// Test multiple users, each with their own coffee machine, with varying operations
test param (nUsers in [1, 2, 3], nOps in [3, 5, 7]) tcMultiUserOperations [main=TestWithMultipleUsers]:
  assert EspressoMachineModesOfOperation in (union EspressoMachine, Users, { TestWithMultipleUsers });

// Test resource constraints with different levels
test param (waterLevel in [0, 25, 50, 100], beanLevel in [0, 25, 50]) tcResourceConstraints [main=TestWithResourceConstraints]:
  assert EspressoMachineModesOfOperation in (union EspressoMachine, Users, { TestWithResourceConstraints });

// BOOLEAN PARAMETER TESTS

// Test boolean configurations
test param (enableSteamer in [true, false], cleaningMode in [true, false]) tcBooleanConfigs [main=TestWithMixedConfiguration]:
  assert EspressoMachineModesOfOperation in (union EspressoMachine, Users, { TestWithMixedConfiguration, SteamerTestUser });

`,LC=`/*
A SaneUser who knows how to use the CoffeeMaker
*/
machine SaneUser {
  var coffeeMakerPanel: CoffeeMakerControlPanel;
  var cups: int;
  start state Init {
    entry (coffeeMaker: CoffeeMakerControlPanel) {
      coffeeMakerPanel = coffeeMaker;
      // inform control panel that I am the user
      send coffeeMakerPanel, eCoffeeMachineUser, this;
      // want to make 2 cups of espresso
      cups = 2;

      goto LetsMakeCoffee;
    }
  }

  state LetsMakeCoffee {
    entry {
      while (cups > 0)
      {
        // lets wait for coffee maker to be ready
        WaitForCoffeeMakerToBeReady();

        // press Espresso button
        PerformOperationOnCoffeeMaker(coffeeMakerPanel, CM_PressEspressoButton);

        // check the status of the machine
        receive {
          case eEspressoCompleted: {
            // lets make the next coffee
            cups = cups - 1;
          }
          case eCoffeeMakerError: (status: tCoffeeMakerState){

            // lets fill the beans or water and reset the machine
            // and go back to making espresso
            PerformOperationOnCoffeeMaker(coffeeMakerPanel, CM_PressResetButton);
          }
        }
      }

      // I am a good user so I will clear the coffee grounds before leaving.
      PerformOperationOnCoffeeMaker(coffeeMakerPanel, CM_ClearGrounds);

      // am done, let me exit
      raise halt;
    }
  }
}

enum tCoffeeMakerOperations {
  CM_PressEspressoButton,
  CM_PressSteamerButton,
  CM_PressResetButton,
  CM_ClearGrounds
}

/*
A crazy user who gets excited by looking at a coffee machine and starts stress testing the machine
by pressing all sorts of random button and opening/closing doors
*/
// TODO: We do not support global constants currently, they can be encoded using global functions.

machine CrazyUser {
  var coffeeMakerPanel: CoffeeMakerControlPanel;
  var numOperations: int;
  start state StartPressingButtons {
    entry (config: (coffeeMaker: CoffeeMakerControlPanel, nOps: int)) {
      var pickedOps: tCoffeeMakerOperations;

      numOperations = config.nOps;
      coffeeMakerPanel = config.coffeeMaker;

      // inform control panel that I am the user
      send coffeeMakerPanel, eCoffeeMachineUser, this;

      while(numOperations > 0)
      {
        pickedOps = PickRandomOperationToPerform();
        PerformOperationOnCoffeeMaker(coffeeMakerPanel, pickedOps);
        numOperations = numOperations - 1;
      }
    }

    // I will ignore all the responses from the coffee maker
    ignore eCoffeeMakerError, eEspressoCompleted, eCoffeeMakerReady;
  }

  // Pick a random enum value (hacky work around)
  // Currently, the choose operation does not support choose over enum value
  fun PickRandomOperationToPerform() : tCoffeeMakerOperations {
    var op_i: int;
    op_i =  choose(3);
    if(op_i == 0)
      return CM_PressEspressoButton;
    else if(op_i == 1)
      return CM_PressSteamerButton;
    else if(op_i == 2)
      return CM_PressResetButton;
    else
      return CM_ClearGrounds;
  }
}

// Specialized user for testing steamer functionality
machine SteamerTestUser
{
  var coffeeMakerPanel: CoffeeMakerControlPanel;
  
  start state Init {
    entry (coffeeMaker: CoffeeMakerControlPanel) {
      coffeeMakerPanel = coffeeMaker;
      send coffeeMakerPanel, eCoffeeMachineUser, this;
      goto TestSteamer;
    }
  }
  
  state TestSteamer {
    entry {
      // Test steamer operations specifically
      PerformOperationOnCoffeeMaker(coffeeMakerPanel, CM_PressSteamerButton);
      
      receive {
        case eCoffeeMakerReady: {
          // Continue with espresso after steamer test
          PerformOperationOnCoffeeMaker(coffeeMakerPanel, CM_PressEspressoButton);
        }
        case eCoffeeMakerError: (status: tCoffeeMakerState) {
          // Reset and try again
          PerformOperationOnCoffeeMaker(coffeeMakerPanel, CM_PressResetButton);
        }
      }
    }
    
    ignore eEspressoCompleted;
  }
}


/* Function to perform an operation on the CoffeeMaker */
fun PerformOperationOnCoffeeMaker(coffeeMakerCP: CoffeeMakerControlPanel, CM_Ops: tCoffeeMakerOperations)
{
  if(CM_Ops == CM_PressEspressoButton) {
    send coffeeMakerCP, eEspressoButtonPressed;
  }
  else if(CM_Ops == CM_PressSteamerButton) {
    send coffeeMakerCP, eSteamerButtonOn;
    // wait for some time and then release the button
    send coffeeMakerCP, eSteamerButtonOff;
  }
  else if(CM_Ops == CM_ClearGrounds)
  {
    send coffeeMakerCP, eOpenGroundsDoor;
    // empty ground and close the door
    send coffeeMakerCP, eCloseGroundsDoor;
  }
  else if(CM_Ops == CM_PressResetButton)
  {
    send coffeeMakerCP, eResetCoffeeMaker;
  }
}

fun WaitForCoffeeMakerToBeReady() {
  receive {
    case eCoffeeMakerReady: {}
    case eCoffeeMakerError: (status: tCoffeeMakerState){ raise halt; }
  }
}
module Users = { SaneUser, CrazyUser };`,DC=`/***************************************************
ReliableFailureDetector is a liveness property to assert that all nodes that have been shutdown
by the failure injector will eventually be detected by the failure detector as a failed node
***************************************************/

spec ReliableFailureDetector observes eNotifyNodesDown, eShutDown {
  // set of nodes that are shutdown by the failure injector but
  // have not been detected.
  var nodesShutdownAndNotDetected: set[Node];
  // set of nodes that have been marked as down by the failure detector
  var nodesDownDetected: set[Node];

  // State where all the nodes that are shutdown by failure injector
  // have been detected by the failure detector
  start state AllShutdownNodesAreDetected {
    on eNotifyNodesDown do (nodes: set[Node])
    {
      var i: int;
      while(i < sizeof(nodes))
      {
        nodesShutdownAndNotDetected -= (nodes[i]);
        nodesDownDetected += (nodes[i]);
        i = i + 1;
      }
    }

    on eShutDown do (node: machine) {
      if(!((node as Node) in nodesDownDetected)) {
        nodesShutdownAndNotDetected += (node as Node);
        goto NodesShutDownButNotDetected;
      }
    }
  }

  // intermediate "unstable" state where the failure detector has not detected all the nodes
  // that have been shutdown by the failure injector
  hot state NodesShutDownButNotDetected {
    on eNotifyNodesDown do (nodes: set[Node])
    {
      var i: int;
      while(i < sizeof(nodes))
      {
        nodesShutdownAndNotDetected -= (nodes[i]);
        nodesDownDetected += (nodes[i]);
        i = i + 1;
      }

      if(sizeof(nodesShutdownAndNotDetected) == 0)
        goto AllShutdownNodesAreDetected; // return to the stable state
    }

    on eShutDown do (node: machine) {
      if(!((node as Node) in nodesDownDetected))
        nodesShutdownAndNotDetected += (node as Node);
    }
  }
}`,OC=`/******************************************
Client machine that keeps track of the alive nodes in the system
*******************************************/

machine Client {
  var myViewOfAliveNodes: set[Node];

  start state Init {
    entry (nodes: set[Node]){
      myViewOfAliveNodes = nodes;
      // do something with the alive nodes
    }

    on eNotifyNodesDown do (dead_nodes: set[Node]){
      var i : int;
      print format("Nodes {0} are down!", dead_nodes);
      while(i < sizeof(dead_nodes))
      {
        myViewOfAliveNodes -= (dead_nodes[i]);
        i = i + 1;
      }
    }
  }
}`,NC=`// event: ping nodes (from failure detector to nodes)
event ePing: (fd: FailureDetector, trial: int);
// event: pong detector (response to ping) (from nodes to failure detector)
event ePong: (node: Node, trial: int);
// event: failure notification to the client (from failure detector to client)
event eNotifyNodesDown: set[Node];

/***************************************************
FailureDetector machine monitors whether a set of nodes in the system are alive (responsive).
It periodically sends ping message to each node and waits for a pong message from the nodes.
The nodes that do not send a pong message after multiple attempts are marked as down or failed
and notified to the client nodes so that they can update their view of the system.
***************************************************/
machine FailureDetector {
  // set of nodes to be monitored
  var nodes: set[Node];
  // set of registered clients
  var clients: set[Client];
  // num of ping attempts made
  var attempts: int;
  // set of alive nodes
  var alive: set[Node];
  // nodes that have responded in the current round
  var respInCurrRound: set[machine];
  // timer to wait for responses from nodes
  var timer: Timer;

  start state Init {
    entry (config: (nodes: set[Node], clients: set[Client])) {
      nodes = config.nodes;
      alive = config.nodes;
      clients = config.clients;
      timer = CreateTimer(this);
      goto SendPingsToAllNodes;
    }
  }

  state SendPingsToAllNodes {
    entry {
      var notRespondedNodes: set[Node];

      if(sizeof(alive) == 0)
        raise halt; // stop myself, no work to do, there are no alive nodes!

      // compute nodes that have not responded with pongs
      notRespondedNodes = PotentiallyDownNodes();
      // send ping events to machines that have not responded in the previous attempt
      UnReliableBroadCast(notRespondedNodes, ePing, (fd =  this, trial = attempts));
      // start wait timer to wait for pong responses
      StartTimer(timer);
    }

    on ePong do (pong: (node: Node, trial: int)) {
      // collect pong responses from alive nodes
      // no need to do any for pong messages from nodes that have marked failed.
      if (pong.node in alive) {
        respInCurrRound += (pong.node);
        if (sizeof(respInCurrRound) == sizeof(alive)) {
          // status of alive nodes has not changed
          CancelTimer(timer);
          goto ResetAndStartAgain;
        }
      }
    }

    on eTimeOut do {
      var nodesDown: set[Node];
      // one more attempt finished
      attempts = attempts + 1;
      // check if there are nodes that have not responded
      if (sizeof(respInCurrRound) < sizeof(alive) ) {
        // maximum number of attempts == 3
        if(attempts < 3) {
          // try again by re-pinging the nodes that have not responded
          goto SendPingsToAllNodes;
        }
        else
        {
          // inform clients about the nodes down
          nodesDown = ComputeNodesDownAndUpdateAliveSet();
          // notification to the client is assumed to be a reliable send so that the client gets an updated view
          ReliableBroadCast(clients, eNotifyNodesDown, nodesDown);
        }
      }
      // lets reset and restart the failure detection
      goto ResetAndStartAgain;
    }
  }

  state ResetAndStartAgain {
    entry {
      // prepare for the next detection phase
      attempts = 0;
      respInCurrRound = default(set[Node]);
      // start timer for inter-phase waiting
      StartTimer(timer);
    }
    on eTimeOut goto SendPingsToAllNodes;
    // detection has finish, these are all delayed pongs and must be ignored
    ignore ePong;
  }

  // compute the potentially down nodes
  // i.e., nodes that have not responded Pong in this round
  fun PotentiallyDownNodes() : set[Node] {
    var i: int;
    var nodesNotResponded: set[Node];
    while (i < sizeof(nodes)) {
      if (nodes[i] in alive && !(nodes[i] in respInCurrRound)) {
          nodesNotResponded += (nodes[i]);
      }
      i = i + 1;
    }
    return nodesNotResponded;
  }

  // compute the nodes that might be down and also update the alive set accordingly
  fun ComputeNodesDownAndUpdateAliveSet() : set[Node] {
    var i: int;
    var nodesDown: set[Node];
    while (i < sizeof(nodes)) {
      if (nodes[i] in alive && !(nodes[i] in respInCurrRound)) {
        alive -= (nodes[i]);
        nodesDown += (nodes[i]);
      }
      i = i + 1;
    }
    return nodesDown;
  }
}



`,BC=`// module to represent the Failure Detector
module FailureDetector = (union {FailureDetector, Node, Client }, Timer);`,_C=`/****************************
Node machine sends a pong message on receiving a ping
*****************************/
machine Node {
  start state WaitForPing {
    on ePing do (req: (fd: FailureDetector, trial: int)) {
      UnReliableSend(req.fd, ePong, (node = this, trial = req.trial));
    }

    on eShutDown do {
      raise halt;
    }
  }
}`,qC=`type tSystemConfig = (
  numNodes: int,
  numClients: int
);

/*****************************************
Setup a system with 3 nodes and 2 clients
******************************************/
machine TestMultipleClients {
  start state Init {
    entry {
      var config: tSystemConfig;
      config = (numNodes = 3, numClients = 2);
      SetupSystemWithFailureInjector(config);
    }
  }
}

param numNodes: int;
param numClients: int;

machine TestWithConfig {
  start state Init {
    entry {
      var config: tSystemConfig;
      config = (numNodes = numNodes, numClients = numClients);
      SetupSystemWithFailureInjector(config);
    }
  }
}

// setup the system for failure detection
fun SetupSystemWithFailureInjector(config: tSystemConfig)
{
  var i : int;
  var nodes: set[Node];
  var clients: set[Client];
  // create Nodes
  while(i < config.numNodes) {
    nodes += (new Node());
    i = i + 1;
  }

  i = 0;
  // create clients
  while(i < config.numClients) {
    clients += (new Client(nodes));
    i = i + 1;
  }
  // create the failure detector
  new FailureDetector((nodes = nodes, clients = clients));

  // create the failure injector
  new FailureInjector((nodes = nodes, nFailures = sizeof(nodes)/2 + 1));
}`,WC=`test tcTest_FailureDetector [main=TestMultipleClients]:
  assert ReliableFailureDetector in
  union { TestMultipleClients }, FailureDetector, FailureInjector;

// Test case that ensures clients don't outnumber nodes for better monitoring distribution
test param (numNodes in [3, 4, 5], numClients in [2, 3, 4]) assume (numClients <= numNodes) tcTest_BalancedLoad [main=TestWithConfig]:
  assert ReliableFailureDetector in
  union { TestWithConfig }, FailureDetector, FailureInjector;`,FC=`// Unreliably send event \`e\` with payload \`p\` to every \`target\`
fun UnreliableBroadcast(target_machines: set[machine], e: event, payload: any) {
	var i: int;
	while (i < sizeof(target_machines)) {
		if (choose()) {
			send target_machines[i], e, payload;
		}
		i = i + 1;
	}
}

// Unreliably send event \`e\` with payload \`p\` to every \`target\`, potentially multiple times
fun UnreliableBroadcastMulti(target_machines: set[machine], e: event, payload: any) {
	var i: int;
	var n: int;
	var k: int;

	while (i < sizeof(target_machines)) {
		// Each message is sent \`k\` is that number of times 
		k = choose(3);
		// Times we've sent the packet so far
		n = 0;
		while (n < k) {		
			send target_machines[i], e, payload;
			n = n + 1;
		}
		i = i + 1;
	}
}

// Reliably send event \`e\` with payload \`p\` to every \`target\`
fun ReliableBroadcast(target_machines: set[machine], e: event, payload: any) {
	var i: int;
	while (i < sizeof(target_machines)) {
		send target_machines[i], e, payload;
		i = i + 1;
	}
}

// Reliably send event \`e\` with payload \`p\` to a majority of \`target\`. Unreliable send to remaining, potentially multiple times.
fun ReliableBroadcastMajority(target_machines: set[machine], e: event, payload: any) {
	var i: int;
	var n: int;
	var k: int;
	var majority: int;

	majority = sizeof(target_machines) / 2 + 1;

	while (i < sizeof(target_machines)) {
		// Each message is sent \`k\` is that number of times 
		k = 1;
		if (i >= majority) {
			k = choose(3);
		}
		// Times we've sent the packet so far
		n = 0;
		while (n < k) {		
			send target_machines[i], e, payload;
			n = n + 1;
		}
		i = i + 1;
	}
}
`,zC=`// This safety spec ensures that the value that was taught never changes (though it can be taught multiple times)
spec OneValueTaught observes eLearn {
	var decided: int;

	start state Init {
		entry {
			decided = -1;
		}

		on eLearn do (payload: (ballot: tBallot, v: tValue)) {
			assert(payload.v != -1);
			if (decided != -1) {
				assert(decided == payload.v);
			}
			decided = payload.v;
		}
	}
}`,jC=`type tBallot = int;
type tValue = int;

type tPrepareReq = (proposer: Proposer, ballot_n: tBallot, v: tValue);
event ePrepareReq: tPrepareReq;
type tPrepareRsp = (acceptor: Acceptor, promised: tBallot, v_accepted: tValue, n_accepted: tBallot);
event ePrepareRsp: tPrepareRsp;
type tAcceptReq = (proposer: Proposer, ballot_n: tBallot, v: tValue);
event eAcceptReq: tAcceptReq;
type tAcceptRsp = (acceptor: Acceptor, accepted: tBallot);
event eAcceptRsp: tAcceptRsp;

// The acceptor role in Paxos (see proposer.p for more details).
machine Acceptor {
	var n_prepared: tBallot;

	var v_accepted: tValue;
	var n_accepted: tBallot;

	start state Init {
		entry {
			n_prepared = -1;
			v_accepted = -1;
			n_accepted = -1;
			goto Accept;
		}
	}

	state Accept {
		// When we get a prepare request, we promise not to accept any prepare requests with a lower ballot number, and return any
		//  proposed value we've accepted, if any.
		on ePrepareReq do (req: tPrepareReq) {
			if (req.ballot_n > n_prepared) {
				send req.proposer, ePrepareRsp, (acceptor = this, promised = req.ballot_n, v_accepted = v_accepted, n_accepted = n_accepted);
				n_prepared = req.ballot_n;
				// print format("{0}: ({1}:{2}, {3})", this, n_accepted, v_accepted, n_prepared);
			}
			// As an optimization, we could send a NACK here to let the proposer know we got its message, allowing it to tell the difference
			//  between losing it's proposal and packet loss. It's just an optimization, so we don't do it yet.
		}

		// Once a proposer has been made the 'leader', it sends us a proposed value. To avoid accepting values from old leaders, we simply
		//  discard any messages with ballot numbers lower than the one we prepared.
		on eAcceptReq do (req: tAcceptReq) {
			if (req.ballot_n >= n_prepared) {
				v_accepted = req.v;
				n_accepted = req.ballot_n;
				// Treat accepting as "prepare, accept" the way that Lamport does in Part Time Parliament (but not in Paxos Made Simple)
				//  See https://brooker.co.za/blog/2021/11/16/paxos.html and https://stackoverflow.com/questions/29880949/contradiction-in-lamports-paxos-made-simple-paper
				n_prepared = req.ballot_n;

				send req.proposer, eAcceptRsp, (acceptor = this, accepted = req.ballot_n);
			}
			// Same story with NACKs here.
		}
	}
}`,VC=`event eLearn: (ballot: tBallot, v: tValue);

// The learner role in Paxos (see proposer.p for more details).
machine Learner {
	var learned_value: tValue;

	start state Init {
		entry {
			learned_value = -1;
			goto Learn;
		}
	}

	state Learn {
		on eLearn do (payload: (ballot: tBallot, v: tValue)) {
			assert(payload.v != -1);
			// This check is a belt-and-braces with the spec, but it's a useful piece of sanity
			assert((learned_value == -1) || (learned_value == payload.v));
			learned_value = payload.v;
		}
	}
}`,HC=`type tProposerConfig = (jury: set[Acceptor], school: set[Learner], value_to_propose: int, proposer_id: int);

machine Proposer {
	var jury: set[Acceptor];
	var school: set[Learner];

	// The ballot number we use for the prepares and accepts that this proposer sends out
	var ballot_n: tBallot;
	var highest_proposal_n: tBallot;
	var value_to_propose: tValue;

	// A number equal to more than half the number of acceptors in the jury
	var majority: int;
	// Acceptors we have received prepare ACKs from. This is a set rather than a counter to deal with the
	//  fact that messages can be delivered multiple times in our network model.
	var prepare_acks: set[Acceptor];
	// Acceptors we have recieved accept ACKs from.
	var accept_acks: set[Acceptor];

	start state Init {
		entry (cfg : tProposerConfig) {
			jury = cfg.jury;
			school = cfg.school;
			// For now, use our id as a ballot number
			ballot_n = cfg.proposer_id;
			value_to_propose = cfg.value_to_propose;
			majority = sizeof(jury) / 2 + 1;
			goto Prepare;
		}
	}

	// Phase 1, prepare
	state Prepare {
		entry {
			var acceptor: Acceptor;
			highest_proposal_n = -1;
			// Step 1a is to fire our proposal at the whole jury
			// ReliableBroadcastMajority(jury, ePrepareReq, (proposer = this, ballot_n = ballot_n, v = value_to_propose));
			foreach (acceptor in jury) {
				send acceptor, ePrepareReq, (proposer = this, ballot_n = ballot_n, v = value_to_propose);
			}
		}

		on ePrepareRsp do (rsp: tPrepareRsp) {
			// The jury then gets back to us, saying whether they have accepted our proposal
			if (rsp.promised == ballot_n) {
				// If this acceptor has already accepted a proposal, we drop the one we're proposing and drive that forward instead.
				if (rsp.n_accepted > highest_proposal_n) {
					highest_proposal_n = rsp.n_accepted;
					value_to_propose = rsp.v_accepted;
				}
				// Add this acceptor to the set of acceptors who have ACKed our proposal
				prepare_acks += (rsp.acceptor);
				// If more than half the acceptors have ACKed our proposal, we've been elected 'leader' and its time to move to phase 2
				if (sizeof(prepare_acks) >= majority) {
					goto Accept;
				}
			}
		}
	}

	// Phase 2, accept
	state Accept {
		entry {
			// We start the accept phase by firing off our accept requests at the jury
			// ReliableBroadcastMajority(jury, eAcceptReq, (proposer = this, ballot_n = ballot_n, v = value_to_propose));
			var acceptor: Acceptor;
			foreach (acceptor in jury) {
				send acceptor, eAcceptReq, (proposer = this, ballot_n = ballot_n, v = value_to_propose);
			}
		}

		// Then get reponses from the acceptors
		on eAcceptRsp do (rsp: tAcceptRsp) {
			if (rsp.accepted == ballot_n) {
				// Add this acceptor to the set that have accepted our value
				accept_acks += (rsp.acceptor);
				// And when more than half agree, it's been decided and it's time to teach
				if (sizeof(accept_acks) >= majority) {
					goto Teach;
				}
			}
		}
		// Stale prepares can be safely ignored
		ignore ePrepareRsp;
	}

	// Phase 3, teaching.
	// We diverge a little from the paper here, which puts the acceptors in the teacher role. Doing it here adds a round-trip,
	//  but makes the learners much simpler (they just accept whatever is thrown at them).
	state Teach {
		entry {
			// ReliableBroadcast(school, eLearn, (v=value_to_propose,));
			var learner: Learner;
			foreach (learner in school) {
				send learner, eLearn, (ballot=ballot_n, v=value_to_propose);
			}
		}

		// At this point it's safe to ignore every message
		ignore eAcceptRsp;
		ignore ePrepareRsp;
	}
}

module Paxos = { Proposer, Acceptor, Learner };
`,UC=`test testBasicPaxos3on5 [main = BasicPaxos3on5]:
	assert OneValueTaught in (union Paxos, { BasicPaxos3on5 });

test testBasicPaxos3on3 [main = BasicPaxos3on3]:
	assert OneValueTaught in (union Paxos, { BasicPaxos3on3 });

test testBasicPaxos3on1 [main = BasicPaxos3on1]:
	assert OneValueTaught in (union Paxos, { BasicPaxos3on1 });

test testBasicPaxos2on3 [main = BasicPaxos2on3]:
	assert OneValueTaught in (union Paxos, { BasicPaxos2on3 });

test testBasicPaxos2on2 [main = BasicPaxos2on2]:
	assert OneValueTaught in (union Paxos, { BasicPaxos2on2 });

test testBasicPaxos1on1 [main = BasicPaxos1on1]:
	assert OneValueTaught in (union Paxos, { BasicPaxos1on1 });

type tPaxosConfig = (n_proposers: int, n_acceptors: int, n_learners: int);

event ePaxosConfig: (quorum: int);

fun SetupPaxos(cfg: tPaxosConfig) {
	var i: int;
	var proposers: set[Proposer];
	var jury: set[Acceptor];
	var school: set[Learner];
	
	var proposerCfg: tProposerConfig;

	announce ePaxosConfig, (quorum = cfg.n_acceptors / 2 + 1,);

	i = 0;
	while (i < cfg.n_acceptors) {
		i = i + 1;
		jury += (new Acceptor());
	}
	i = 0;
	while (i < cfg.n_learners) {
		i = i + 1;
		school += (new Learner());
	}
	i = 0;
	while (i < cfg.n_proposers) {
		i = i + 1;
		proposerCfg = (jury = jury, school = school, value_to_propose = i + 100 + choose(50), proposer_id = i + choose(50));
		proposers += (new Proposer(proposerCfg));
	}
}

machine BasicPaxos3on5 {
	start state Init {
		entry {
			var config: tPaxosConfig;
			config = (n_proposers = 3, n_acceptors = 5, n_learners = 1);
			SetupPaxos(config);
		}
	}
}

machine BasicPaxos3on3 {
	start state Init {
		entry {
			var config: tPaxosConfig;
			config = (n_proposers = 3, n_acceptors = 3, n_learners = 1);
			SetupPaxos(config);
		}
	}
}

machine BasicPaxos3on1 {
	start state Init {
		entry {
			var config: tPaxosConfig;
			config = (n_proposers = 3, n_acceptors = 1, n_learners = 1);
			SetupPaxos(config);
		}
	}
}

machine BasicPaxos2on3 {
	start state Init {
		entry {
			var config: tPaxosConfig;
			config = (n_proposers = 2, n_acceptors = 3, n_learners = 1);
			SetupPaxos(config);
		}
	}
}

machine BasicPaxos2on2 {
	start state Init {
		entry {
			var config: tPaxosConfig;
			config = (n_proposers = 2, n_acceptors = 2, n_learners = 1);
			SetupPaxos(config);
		}
	}
}

machine BasicPaxos1on1 {
	start state Init {
		entry {
			var config: tPaxosConfig;
			config = (n_proposers = 1, n_acceptors = 1, n_learners = 1);
			SetupPaxos(config);
		}
	}
}

machine BasicPaxos4on4 {
	start state Init {
		entry {
			var config: tPaxosConfig;
			config = (n_proposers = 4, n_acceptors = 4, n_learners = 1);
			SetupPaxos(config);
		}
	}
}
`,$C=`spec LivenessClientsDone observes eClientPutRequest, eClientGetRequest, eClientFinishedMonitor {
    var activeClients: set[Client];
    var finishedClients: set[Client];
    
    start state Init {
        entry {
            activeClients = default(set[Client]);
            finishedClients = default(set[Client]);
            goto AllClientsDone;
        }
    }

    hot state ClientsActive {
        on eClientFinishedMonitor do (c: Client) {
            finishedClients += (c);
            if (finishedClients == activeClients) {
                goto AllClientsDone;
            }
        }

        // on eClientRequest do (payload: tClientRequest) {
        //     if (payload.client == payload.sender) {
        //         activeClients += (payload.client);
        //     }
        // }

        on eClientPutRequest do (payload: tClientPutRequest) {
            if (payload.client == payload.sender) {
                activeClients += (payload.client);
            }
        }

        on eClientGetRequest do (payload: tClientGetRequest) {
            if (payload.client == payload.sender) {
                activeClients += (payload.client);
            }
        }
    }

    cold state AllClientsDone {
        on eClientPutRequest do (payload: tClientPutRequest) {
            if (payload.client == payload.sender) {
                activeClients += (payload.client);
                goto ClientsActive;
            }
        }

        on eClientGetRequest do (payload: tClientGetRequest) {
            if (payload.client == payload.sender) {
                activeClients += (payload.client);
                goto ClientsActive;
            }
        }

        on eClientFinishedMonitor do (c: Client) {
            print format("Problematic: {0}", c);
            assert false, "Received a client finished event after all clients are done.";
        }
    }
}`,GC=`type tRequestMetadata = (client: machine, transId: int);

spec LivenessProgress observes eClientWaitingResponse, eClientGotResponse {
    var clientRequests: set[tRequestMetadata];

    start state Init {
        entry {
            clientRequests = default(set[tRequestMetadata]);
            goto Done;
        }
    }

    hot state PendingRequestsExist {

        on eClientWaitingResponse do (payload: tRequestMetadata) {
            clientRequests += ((client=payload.client, transId=payload.transId));
        }

        on eClientGotResponse do (payload: tRequestMetadata) {
            clientRequests -= ((client=payload.client, transId=payload.transId));
            if (sizeof(clientRequests) == 0) {
                goto Done;
            }
        }

    }

    cold state Done {
        on eClientWaitingResponse do (payload: tRequestMetadata) {
            clientRequests += ((client=payload.client, transId=payload.transId));
            goto PendingRequestsExist;
        }
    }
}`,KC=`spec SafetyLeaderCompleteness observes eBecomeLeader {
    var committedLogs: set[tServerLog];

    start state Init {
        entry {
            committedLogs = default(set[tServerLog]);
            goto MonitoringLeaderChange;
        }
    }

    state MonitoringLeaderChange {
        on eBecomeLeader do (payload: (term:int, leader:Server, log: seq[tServerLog], commitIndex: int)) {
            // check all already-committed logs is remain committed in the new leader's log
            var i: int;
            var e: tServerLog;
            foreach (e in committedLogs) {
                assert e in payload.log, format("{0} was committed before but not in the new leader's log", e);
            }
            // add the committed logs
            i = 0;
            while (i <= payload.commitIndex) {
                committedLogs += (payload.log[i]);
                i = i + 1;
            }
        }
    }
}`,QC=`spec SafetyLogMatching observes eNotifyLog {
    var allLogs: map[Server, seq[tServerLog]];

    start state MonitorLogUpdates {
        entry {
            allLogs = default(map[Server, seq[tServerLog]]);
        }

        on eNotifyLog do (payload: (timestamp: int, server: Server, log: seq[tServerLog])) {
            var i: int;
            var j: int;
            var s1: Server;
            var s2: Server;
            if (!(payload.server in allLogs)) {
                allLogs[payload.server] = payload.log;
            }
            i = 0;
            while (i < sizeof(keys(allLogs))) {
                s1 = keys(allLogs)[i];
                j = i + 1;
                while (j < sizeof(keys(allLogs))) {
                    s2 = keys(allLogs)[j];
                    if (sizeof(allLogs[s1]) > sizeof(allLogs[s2])) {
                        assert checkLogMatching(allLogs[s2], allLogs[s1]);
                    } else {
                        assert checkLogMatching(allLogs[s1], allLogs[s2]);
                    }
                    j = j + 1;
                }
                i = i + 1;
            }
        }
    }

    fun checkLogMatching(xs: seq[tServerLog], ys: seq[tServerLog]): bool {
        var i: int;
        var highestMatch: int;
        var logsA: seq[tServerLog];
        var logsB: seq[tServerLog];
        if (sizeof(xs) > sizeof(ys)) {
            logsA = ys;
            logsB = xs;
        } else {
            logsA = xs;
            logsB = ys;
        }
        i = sizeof(logsA) - 1;
        while (i >= 0 && logsA[i] != logsB[i]) {
           i = i - 1;
        }
        while (i >= 0) {
            if (logsA[i] != logsB[i]) {
                return false;
            }
            i = i - 1;
        }
        return true;
    }
}
`,YC=`type tBecomeLeader = (term:TermId, leader:Server, log: seq[tServerLog], commitIndex: LogIndex);
event eBecomeLeader: tBecomeLeader;

spec SafetyOneLeader observes eBecomeLeader{
    var termToLeader: map[int, Server];
    start state Init {
        entry {
            termToLeader = default(map[int, Server]);
        }
        on eBecomeLeader do (payload: (term:int, leader:Server, log: seq[tServerLog], commitIndex: int)) {
            if (payload.term in keys(termToLeader)) {
                assert termToLeader[payload.term] == payload.leader, format("At term {0} there are multiple leaders: {1} and {2}.", payload.term, termToLeader[payload.term], payload.leader);
            }
            termToLeader += (payload.term, payload.leader);
        }
    }
}`,XC=`event eEntryApplied: (logIndex: LogIndex, term: TermId, key: KeyT, value: ValueT, transId: TransId, client: Client);

spec SafetyStateMachine observes eEntryApplied {
    var appliedEntries: map[int, tServerLog];
    start state MonitorEntryApplied {
        entry {
            appliedEntries = default(map[int, tServerLog]);
        }

        on eEntryApplied do (payload: (logIndex: LogIndex, term: TermId, key: KeyT, value: ValueT, transId: TransId, client: Client)) {
            var log: tServerLog;
            log = (term=payload.term, key=payload.key, value=payload.value, client=payload.client, transId=payload.transId);
            if (!(payload.logIndex in keys(appliedEntries))) {
                appliedEntries[payload.logIndex] = log;
            }
            assert appliedEntries[payload.logIndex] == log,
                format("Entry@{0}={1} applied before does not match with {2}", payload.logIndex, appliedEntries[payload.logIndex], log);
        }
    }
} `,JC=`spec SafetySynchronization observes eClientPutRequest, eClientGetRequest, eRaftGetResponse, eRaftPutResponse, eBecomeLeader {
    var localKVStore: KVStore;
    var requestResultMap: map[Client, map[int, Result]];
    var getRequestMap: map[Client, map[TransId, KeyT]];
    var putRequestMap: map[Client, map[TransId, (key: KeyT, value: ValueT)]];
    var seenId: map[Client, set[int]];
    var respondedId : map[Client, set[int]];
    var currentLeader: Server;
    var term: TermId;

    start state Init {
        entry {
            localKVStore = newStore();
            requestResultMap = default(map[Client, map[int, Result]]);
            getRequestMap = default(map[Client, map[TransId, KeyT]]);
            putRequestMap = default(map[Client, map[TransId, (key: KeyT, value: ValueT)]]);
            seenId = default(map[Client, set[int]]);
            respondedId = default(map[Client, set[int]]);
            currentLeader = default(Server);
            term = -1;
            goto Listening;
        }
    }

    state Listening {
        on eBecomeLeader do (payload: tBecomeLeader) {
            if (payload.term > term) {
                currentLeader = payload.leader;
                term = payload.term;
            }
        }

        on eClientGetRequest do (payload: tClientGetRequest) {
            if (!(payload.client in keys(seenId))) {
                seenId[payload.client] = default(set[int]);
            }
            if (!(payload.client in keys(respondedId))) {
                respondedId[payload.client] = default(set[int]);
            }
            if (!(payload.client in keys(getRequestMap))) {
                getRequestMap[payload.client] = default(map[TransId, KeyT]);
            }
            if (payload.client == payload.sender && !(payload.transId in seenId[payload.client])) {
                seenId[payload.client] += (payload.transId);
            }
            getRequestMap[payload.client][payload.transId] = payload.key;
        }

        on eClientPutRequest do (payload: tClientPutRequest) {
            if (!(payload.client in keys(seenId))) {
                seenId[payload.client] = default(set[int]);
            }
            if (!(payload.client in keys(respondedId))) {
                respondedId[payload.client] = default(set[int]);
            }
            if (!(payload.client in keys(putRequestMap))) {
                putRequestMap[payload.client] = default(map[TransId, (key: KeyT, value: ValueT)]);
            }
            if (payload.client == payload.sender && !(payload.transId in seenId[payload.client])) {
                seenId[payload.client] += (payload.transId);
            }
            putRequestMap[payload.client][payload.transId] = (key=payload.key, value=payload.value);
        }

        on eRaftGetResponse do (payload: tRaftGetResponse) {
            var execResult: ExecutionResult;
            checkResponseValid(payload.client, payload.sender, payload.transId);
            if (!(payload.client in keys(respondedId)) && currentLeader == payload.sender) {
                respondedId[payload.client] += (payload.transId);
                execResult = executeGet(localKVStore, getRequestMap[payload.client][payload.transId]);
                assert execResult.result.success == payload.success, format("Inconsistent status: {0}", getRequestMap[payload.client][payload.transId]);
                assert execResult.result.value == payload.value, format("Inconsistent Get result! Expected {0}, got {1}", execResult.result.value, payload.value);
            }
        }

        on eRaftPutResponse do (payload: tRaftPutResponse) {
            var execResult: ExecutionResult;
            checkResponseValid(payload.client, payload.sender, payload.transId);
            if (!(payload.client in keys(respondedId)) && currentLeader == payload.sender) {
                respondedId[payload.client] += (payload.transId);
                assert payload.client in keys(putRequestMap);
                execResult = executePut(localKVStore, putRequestMap[payload.client][payload.transId].key, putRequestMap[payload.client][payload.transId].value);
                localKVStore = execResult.newState;
            }
        }
    }
    fun checkResponseValid(client: Client, sender: Server, tId: TransId) {
        assert currentLeader != default(Server), "Leader is un-initialized but got an eRaftResponse!";
        if (currentLeader == sender) {
            assert client in keys(seenId), format("Responding to a client that has not sent any request: {0}", client);
            assert tId in seenId[client], format("Responding to a non-existing transactionId {0} sent by {1}", tId, client);
        }
    }
}
`,ZC=`enum Op { 
    PUT,
    GET
}
type KeyT = int;
type ValueT = int;
type Command = (op: Op, key: KeyT, value: ValueT);
type Result = (success: bool, value: ValueT);
type ExecutionResult = (newState: KVStore, result: Result);
type KVStore = map[KeyT, ValueT];

fun IsPut(op: Op): bool {
    return op == PUT;
}

fun IsGet(op: Op): bool {
    return op == GET;
}

fun newStore(): KVStore {
    return default(KVStore);
}

fun executeGet(store: KVStore, key: KeyT): ExecutionResult {
    if (!(key in store)) {
        return (newState=store, result=(success=false, value=-1));
    }
    return (newState=store, result=(success=true, value=store[key]));
}

fun executePut(store: KVStore, key: KeyT, value: ValueT): ExecutionResult {
    store[key] = value;
    return (newState=store, result=(success=true, value=-1));
}
`,eb=`/*
* A client that uses the Raft cluster.
* The client sends a sequence of commands to the cluster.
* It does not move on the next command until it receives a response from the cluster.
* The client retries sending the command if a certain amount of heartbeats timeouts occur.
*/

// Client requests
type tClientRequest = (transId: TransId, client: Client, cmd: Command, sender: Client);
type tClientPutRequest = (transId: TransId, client: Client, key: KeyT, value: ValueT, sender: Client);
type tClientGetRequest = (transId: TransId, client: Client, key: KeyT, sender: Client);
event eClientPutRequest: tClientPutRequest;
event eClientGetRequest: tClientGetRequest;
// event eClientRequest: tClientRequest;
// The event of notifying the monitor that the client is waiting for a response
event eClientWaitingResponse: (client: Client, transId: int);
// The event of notifying the monitor that the client got a response for a transaction
event eClientGotResponse: (client: Client, transId: int);
// The event of notifying the monitor that the client finished
event eClientFinishedMonitor: Client;
// The event of notifying the view service that the client finished
event eClientFinished: Client;

machine Client {
    var worklist: seq[Command];
    var servers: set[machine];
    var ptr: int;
    var tId: int;
    var retries: int;
    var currentCmd: Command;
    var view: View;
    var timer: Timer;

    start state Init {
        entry (config: (viewService: View, servers: set[machine], requests: seq[Command])) {
            worklist = config.requests;
            servers = config.servers;
            view = config.viewService;
            ptr = 0;
            tId = 0;
            timer = new Timer((user=this, timeoutEvent=eHeartbeatTimeout));
            goto SendOne;
        }
    }

    state SendOne {
        entry {
            var cmd: Command;
            // print format("{0} is at {1}", this, ptr);
            // print format("Worklist {0}", worklist);
            if (sizeof(worklist) == ptr) {
                // if no more work to do, go to Done
                goto Done;
            } else {
                // get the current command and increase transaction id
                currentCmd = worklist[ptr];
                ptr = ptr + 1;
                tId = tId + 1;
                broadcastToCluster();
                goto WaitForResponse;
            }
        }
    }

    state WaitForResponse {
        entry {
            announce eClientWaitingResponse, (client=this, transId=tId);
            startTimer(timer);
            retries = 0;
        }

        on eHeartbeatTimeout do {
            // print format("Client {0} timed out waiting for response {1}; current retries: {2}", this, tId, retries / 50);
            if (retries % 200 == 0) {
                // retries every 200 heartbeats
                broadcastToCluster();
            }
            retries = retries + 1;
            startTimer(timer);
        }

        on eRaftGetResponse do (resp: tRaftGetResponse) {
            handleResponse(resp.transId);
        }

        on eRaftPutResponse do (resp: tRaftPutResponse) {
            handleResponse(resp.transId);
        }
    }

    fun handleResponse(responseTransId: TransId) {
        if (responseTransId == tId) {
            announce eClientGotResponse, (client=this, transId=tId);
            retries = 0;
            goto SendOne;
        }
    }

    fun broadcastToCluster() {
        var s: machine;
        foreach (s in servers) {
            if (currentCmd.op == GET) {
                send s, eClientGetRequest, (transId=tId, client=this, key=currentCmd.key, sender=this);
            } else {
                send s, eClientPutRequest, (transId=tId, client=this, key=currentCmd.key, value=currentCmd.value, sender=this);
            }
        }
    }

    state Done {
        entry {
            announce eClientFinishedMonitor, this;
            send view, eClientFinished, this;
        }
        ignore eRaftGetResponse, eRaftPutResponse, eHeartbeatTimeout;
    }
}`,tb=`module Server = { Server };
module Timer = { Timer };
module Client = { Client };
module View = { View };`,nb=`/*****************************************************************************
* A node in the Raft cluster.
* The implementation follows the Raft paper (Ongaro and Ousterhout, 2014).
* This implementation does not include log compaction / cluster reconfiguration.
******************************************************************************/
type ServerId = int;
type TransId = int;

// Response message to the client
type tRaftResponse = (sender: Server, client: Client, transId: TransId, result: Result);
// event eRaftResponse: tRaftResponse;
type tRaftPutResponse = (sender: Server, client: Client, transId: TransId, key: KeyT);
type tRaftGetResponse = (sender: Server, client: Client, success: bool, transId: TransId, value: ValueT);
// type tRaftGetError = (sender: Server, client: Client, transId: TransId, key: KeyT);
event eRaftPutResponse: tRaftPutResponse;
event eRaftGetResponse: tRaftGetResponse;
// event eRaftGetError: tRaftGetError;

event ePutReq: (client: Client, transId: TransId, key: KeyT, value: ValueT);
event eGetReq: (client: Client, transId: TransId, key: KeyT);
event ePutResp: (client: Client, transId: TransId, key: KeyT, value: ValueT);
event eGetResp: (client: Client, transId: TransId, success: bool, key: KeyT, value: ValueT);

// Server initialization message
event eServerInit: (myId: ServerId, cluster: set[Server], viewServer: View);

// Vote request and response
type TermId = int;
type LogIndex = int;
type tRequestVote = (term: TermId, candidate: Server, lastLogIndex: LogIndex, lastLogTerm: TermId);
event eRequestVote: tRequestVote;
type tRequestVoteReply = (sender: Server, term: TermId, voteGranted: bool);
event eRequestVoteReply: tRequestVoteReply;

// Heartbeat message and response
type tAppendEntries = (term: TermId, leader: Server, prevLogIndex: LogIndex,
                       prevLogTerm: TermId, entries: seq[tServerLog], leaderCommit: LogIndex);

event eAppendEntries: tAppendEntries;
type tAppendEntriesReply = (sender: Server, term: TermId, success: bool, matchedIndex: LogIndex);
event eAppendEntriesReply: tAppendEntriesReply;

// Resetting a server (to model crashes)
event eReset;

// events to notify the server role change to the view server
event eViewChangedFollower: Server;
event eViewChangedLeader: Server;
event eViewChangedCandidate: Server;

// server logs
type tServerLog = (term: TermId, key: KeyT, value: ValueT, client: Client, transId: TransId);

fun TermLT(t1: TermId, t2: TermId): bool {
    return t1 < t2;
}

fun IsSuccess(s: bool): bool {
    return s;
}

fun IsFailure(s: bool): bool {
    return !s;
}

// A node in the Raft cluster
machine Server {
    var serverId: ServerId;
    // the application state
    var kvStore: KVStore;
    // the leader of the cluster
    var leader: Server;
    // size of the cluster
    var clusterSize: int;
    // nodes in the cluster
    var peers: set[Server];
    // the view service
    var viewServer: View;

    // Leader state (volatile)
    var nextIndex: map[Server, int];
    var matchIndex: map[Server, int];
    // Leader state (persistent)
    var currentTerm: int;
    var logs: seq[tServerLog];

    // States for voting
    var votedFor: Server;
    var votesReceived: set[Server];

    // all servers
    var commitIndex: int;
    var lastApplied: int;

    var role: string;
    var logNotifyTimestamp: int;

    // var heartbeatTimer: Timer;
    // var clientRequestQueue: seq[tClientRequest];
    var requestSeqNum: int;
    var clientPutRequestQueue: seq[(seqNum: int, payload: tClientPutRequest)];
    var clientGetRequestQueue: seq[(seqNum: int, payload: tClientGetRequest)];
    var clientPutRequestCache: map[Client, map[int, tRaftPutResponse]];
    var clientGetRequestCache: map[Client, map[int, tRaftGetResponse]];

    start state Init {
        entry {}
        on eServerInit do (setup: (myId: ServerId, cluster: set[Server], viewServer: View)) {
            kvStore = newStore();
            serverId = setup.myId;
            assert this in setup.cluster, "Server should be in the cluster";
            clusterSize = sizeof(setup.cluster);
            peers = setup.cluster;
            nextIndex = default(map[Server, int]);
            matchIndex = default(map[Server, int]);
            votesReceived = default(set[Server]);
            logs = default(seq[tServerLog]);
            // clientRequestQueue = default(seq[tClientRequest]);
            clientPutRequestQueue = default(seq[(seqNum: int, payload: tClientPutRequest)]);
            clientGetRequestQueue = default(seq[(seqNum: int, payload: tClientGetRequest)]);
            requestSeqNum = 0;
            clientPutRequestCache = default(map[Client, map[int, tRaftPutResponse]]);
            clientGetRequestCache = default(map[Client, map[int, tRaftGetResponse]]);
            viewServer = setup.viewServer;

            currentTerm = 0;
            logNotifyTimestamp = 0;
            votedFor = default(Server);
            commitIndex = -1;
            lastApplied = -1;
            leader = default(Server);
            role = "Uninitialized";
            goto Follower;
        }
        ignore eReset, eElectionTimeout, eHeartbeatTimeout;
    }

    state Follower {
        entry {
            role = "F";
            // reset the leader upon being a follower
            leader = default(Server);
            printLog("Become follower");
            send viewServer, eViewChangedFollower, this;
        }

        on eShutdown do {
            goto FinishedServing;
        }

        on eReset do {
            reset();
        }

        on eRequestVote do (payload: tRequestVote) {
            handleRequestVote(payload);
        }

        on eClientPutRequest do (payload: tClientPutRequest) {
            // check if the request has been processed
            respondOrForwardPut(payload);
        }

        on eClientGetRequest do (payload: tClientGetRequest) {
            respondOrForwardGet(payload);
        }

        on eAppendEntries do (payload: tAppendEntries) {
            if (payload.term >= currentTerm) {
                printLog(format("heartbeat updated term from {0} to {1}", currentTerm, payload.term));
                if (leader != default(Server) && payload.leader != leader) {
                    votedFor = default(Server);
                }
                leader = payload.leader;
                updateTermAndVote(payload.term);
            }
            handleAppendEntries(payload);
            if (leader != default(Server)) {
                // propagate client requests to the leader
                drainRequestQueuesTo(leader);
            }
        }

        on eElectionTimeout do {
            printLog("Election timeout");
            goto Candidate;
        }

        on eHeartbeatTimeout do {
            notifyViewLog();
        }

        ignore eRequestVoteReply, eAppendEntriesReply;
    }

    state Candidate {
        entry {
            var peer: Server;
            var lastTerm: int;
            role = "C";
            printLog("Become candidate");
            send viewServer, eViewChangedCandidate, this;
            currentTerm = currentTerm + 1;
            votedFor = default(Server);
            votesReceived = default(set[Server]);
            votesReceived += (this);
            // if there is only 1 server, no need to request vote
            if (sizeof(votesReceived) > clusterSize / 2) {
                goto Leader;
            } else {
                printLog(format("RequestVote with term {0}", currentTerm));
                broadcastRequest(this, peers, eRequestVote,
                                    (term=currentTerm,
                                        candidate=this,
                                        lastLogIndex=lastLogIndex(logs),
                                        lastLogTerm=lastLogTerm(logs)));
            }
        }

        on eClientPutRequest do (payload: tClientPutRequest) {
            if (checkClientRequestCache(payload.client, payload.transId)) {
                return;
            }
            clientPutRequestQueue += (sizeof(clientPutRequestQueue), (seqNum=requestSeqNum, payload=payload));
            requestSeqNum = requestSeqNum + 1;
        }

        on eShutdown do {
            goto FinishedServing;
        }

        on eAppendEntries do (payload: tAppendEntries) {
            if (payload.term >= currentTerm) {
                // if there is a leader, convert to follower
                printLog(format("Received heartbeat from a leader with higher term {0}", payload.term));
                updateTermAndVote(payload.term);
                handleAppendEntries(payload);
                goto Follower;
            } else {
                printLog(format("Received smaller term heartbeat from {0}", payload.leader));
                handleAppendEntries(payload);
            }
        }

        on eAppendEntriesReply do (payload: tAppendEntriesReply) {
            if (payload.term > currentTerm) {
                printLog(format("Received AppendEntriesReply with higher term {0}", payload.term));
                updateTermAndVote(payload.term);
                goto Follower;
            }
        }

        on eRequestVoteReply do (payload: tRequestVoteReply) {
            if (payload.term > currentTerm) {
                // if there is a vote with higher term, convert to follower
                updateTermAndVote(payload.term);
                goto Follower;
            } else if (payload.voteGranted && payload.term == currentTerm) {
                // if the vote is granted for me in this term, count it.
                votesReceived += (payload.sender);
                printLog(format("vote granted by {0}; now vote={1}", payload.sender, votesReceived));
                if (sizeof(votesReceived) > clusterSize / 2) {
                    // if majority votes are received, become a leader
                    printLog(format("majority votes: {0}", votesReceived));
                    goto Leader;
                }
            } else {
                printLog(format("Received rejection/invalid reply: {0}", payload));
            }
        }
        
        on eRequestVote do (payload: tRequestVote)  {
            if (payload.term > currentTerm) {
                updateTermAndVote(payload.term);
                handleRequestVote(payload);
                goto Follower;
            } else {
                send payload.candidate, eRequestVoteReply, (sender=this, term=currentTerm, voteGranted=false);
            }
        }

        on eReset do {
            reset();
        }

        on eElectionTimeout do {
            goto Candidate;
        }

        on eHeartbeatTimeout do {
            notifyViewLog();
        }

        ignore eClientGetRequest;
    }

    state Leader {
        entry {
            role = "L";
            send viewServer, eViewChangedLeader, this;
            leader = this;
            // instantiate the nextIndex and matchIndex
            nextIndex = fillMap(this, nextIndex, peers, sizeof(logs));
            matchIndex = fillMap(this, matchIndex, peers, -1);
            announce eBecomeLeader, (term=currentTerm, leader=this, log=logs, commitIndex=commitIndex);
            // first exhaust the client request queu on its own
            drainRequestQueuesTo(this);
            printLog(format("Become leader with Log={0}, commiteIndex={1}, lastApplied={2}", logs, commitIndex, lastApplied));
            broadcastAppendEntries();
        }

        on eShutdown do {
            goto FinishedServing;
        }

        on eRequestVote do (payload: tRequestVote) {
            if (currentTerm < payload.term) {
                printLog(format("Saw higher term as a leader: currentTerm={0}, payload.term={1}", currentTerm, payload.term));
                handleRequestVote(payload);
                becomeFollower(payload.term);
            } else {
                printLog(format("Reject proposal from {0} with term {1}", payload.candidate, payload.term));
                send payload.candidate, eRequestVoteReply, (sender=this, term=currentTerm, voteGranted=false);
            }
        }

        on eRequestVoteReply do (payload: tRequestVoteReply) {
            if (payload.term > currentTerm) {
                printLog(format("received RequestVoteReply with higher term {0}", payload.term));
                updateTermAndVote(payload.term);
                goto Follower;
            }
        }

        on eHeartbeatTimeout do {
            broadcastAppendEntries();
            leaderCommits();
            notifyViewLog();
        }

        on eAppendEntries do (payload: tAppendEntries) {
            if (payload.term > currentTerm) {
                // if the leader receives an AppendEntries from another leader with a higher term
                // step down to a follower
                printLog(format("received AppendEntries with higher term {0}; handle then step down", payload.term));
                updateTermAndVote(payload.term);
                handleAppendEntries(payload);
                becomeFollower(payload.term);
            } else if (payload.term < currentTerm) {
                send payload.leader, eAppendEntriesReply, (sender=this, term=currentTerm, success=false, matchedIndex=-1);
                leaderCommits();
            }
        }

        on eAppendEntriesReply do (payload: tAppendEntriesReply) {
            printLog(format("Leader({0}, {1}) received AppendEntriesReply from {2}: {3}", this, currentTerm, payload.sender, payload));
            if (payload.term < currentTerm) {
                printLog("Ignore AppendEntriesReply with outdated term");
                return;
            }
            if (payload.term > currentTerm) {
                // if the leader receives a reply from another leader with a higher term
                // step down to a follower
                printLog(format("received reply from another leader with term {0} > currentTerm {1}", payload.term, currentTerm));
                updateTermAndVote(payload.term);
                goto Follower;
            }
            if (payload.success) {
                // if the heartbeat is accepted,
                // update the nextIndex and matchIndex for the corresponding node.
                printLog(format("entries accepted by {0}; matchIndex={1}", payload.sender, payload.matchedIndex));
                // Note that the leader can receive delayed message, so here we need to keep the
                // larger values of nextIndex and matchIndex.
                nextIndex[payload.sender] = Max(nextIndex[payload.sender], payload.matchedIndex + 1);
                matchIndex[payload.sender] = Max(matchIndex[payload.sender], payload.matchedIndex);
                leaderCommits();
            } else {
                if (payload.matchedIndex < 0) {
                    // a rejection of me as a leader in an earlier term
                    // this can happen when me and the other node simultaneously become a candidate
                    // the other node received my heartbeat from the previous term and sent me a rejection
                    // but currently, I am already at the newer term, so I can safely ignore this rejection
                    printLog(format("outdated rejection with term={0}, matchedIndex={1}", payload.term, payload.matchedIndex));
                    return;
                }
                // rejected because of log mismatch
                // now, payload.matchedIndex is the index of the (potentially) first mismatched term
                printLog(format("re-sync with {0} at index {1}", payload.sender, payload.matchedIndex));
                // still, the leader can receive outdated message. To be conservative, it is safe to synchronize from
                // a smaller index.
                nextIndex[payload.sender] = Min(payload.matchedIndex, nextIndex[payload.sender]);
            }
        }

        on eClientGetRequest do (payload: tClientGetRequest) {
            var execResult: ExecutionResult;
            if (checkClientRequestCache(payload.client, payload.transId)) {
                return;
            }
            announce eGetReq, (client=payload.client, transId=payload.transId, key=payload.key);
            execResult = executeGet(kvStore, payload.key);
            send payload.client, eRaftGetResponse, (sender=this, client=payload.client, success=execResult.result.success, transId=payload.transId, value=execResult.result.value);
            announce eGetResp, (client=payload.client, transId=payload.transId, success=execResult.result.success, key=payload.key, value=execResult.result.value);
            if (!(payload.client in keys(clientGetRequestCache))) {
                clientGetRequestCache[payload.client] = default(map[int, tRaftGetResponse]);
            }
            clientGetRequestCache[payload.client][payload.transId] = (sender=this, client=payload.client, success=execResult.result.success, transId=payload.transId, value=execResult.result.value);
        }

        on eClientPutRequest do (payload: tClientPutRequest) {
            var newEntry: tServerLog;
            var target: Server;
            var entries: seq[tServerLog];
            var i: int;
            // print format("Received client request {0}", payload);
            if (checkClientRequestCache(payload.client, payload.transId)) {
                return;
            }
            newEntry = (term=currentTerm, key=payload.key, value=payload.value, client=payload.client, transId=payload.transId);
            if (!(newEntry in logs)) {
                logs += (sizeof(logs), newEntry);
                announce ePutReq, (client=payload.client, transId=payload.transId, key=payload.key, value=payload.value);
            }
            printLog(format("Current leader logs={0}", logs));
            printLog(format("nextIndex={0}, matchIndex={1}", nextIndex, matchIndex));
            // use info of nextIndex and matchIndex to broadcast to peers
            foreach (target in peers) {
                if (target != this && nextIndex[target] < sizeof(logs)) {
                    entries = default(seq[tServerLog]);
                    i = nextIndex[target];
                    while (i < sizeof(logs)) {
                        entries += (i - nextIndex[target], logs[i]);
                        i = i + 1;
                    }
                    send target, eAppendEntries, (term=currentTerm,
                                                leader=this,
                                                prevLogIndex=nextIndex[target] - 1,
                                                prevLogTerm=getLogTerm(logs, nextIndex[target] - 1),
                                                entries=entries,
                                                leaderCommit=commitIndex);
                }
            }
            leaderCommits();
        }

        on eReset do {
            reset();
        }

        ignore eElectionTimeout;
    }

    state FinishedServing {
        entry {
            printLog("Service ended. Shutdown.");
        }
        ignore eShutdown, eRequestVote, eRequestVoteReply, eAppendEntries, eAppendEntriesReply, eClientPutRequest, eClientGetRequest, eHeartbeatTimeout, eElectionTimeout;
    }

    fun drainRequestQueuesTo(target: Server) {
        while (sizeof(clientPutRequestQueue) > 0 || sizeof(clientGetRequestQueue) > 0) {
            if (sizeof(clientPutRequestQueue) == 0) {
                send target, eClientGetRequest, forwardedGetRequest(clientGetRequestQueue[0].payload);
                clientGetRequestQueue -= (0);
            } else if (sizeof(clientGetRequestQueue) == 0) {
                send target, eClientPutRequest, forwardedPutRequest(clientPutRequestQueue[0].payload);
                clientPutRequestQueue -= (0);
            } else {
                if (clientPutRequestQueue[0].seqNum < clientGetRequestQueue[0].seqNum) {
                    send target, eClientPutRequest, forwardedPutRequest(clientPutRequestQueue[0].payload);
                    clientPutRequestQueue -= (0);
                } else {
                    send target, eClientGetRequest, forwardedGetRequest(clientGetRequestQueue[0].payload);
                    clientGetRequestQueue -= (0);
                }
            }
        }
    }

    fun respondOrForwardPut(payload: tClientPutRequest) {
        if (checkClientRequestCache(payload.client, payload.transId)) {
            return;
        }
        if (leader != default(Server)) {
            send leader, eClientPutRequest, forwardedPutRequest(payload);
        } else {
            clientPutRequestQueue += (sizeof(clientPutRequestQueue), (seqNum=requestSeqNum, payload=payload));
            requestSeqNum = requestSeqNum + 1;
        }
    }

    fun respondOrForwardGet(payload: tClientGetRequest) {
        if (checkClientRequestCache(payload.client, payload.transId)) {
            return;
        }
        if (leader != default(Server)) {
            send leader, eClientGetRequest, forwardedGetRequest(payload);
        } else {
            clientGetRequestQueue += (sizeof(clientGetRequestQueue), (seqNum=requestSeqNum, payload=payload));
            requestSeqNum = requestSeqNum + 1;
        }
    }

    fun forwardedPutRequest(req: tClientPutRequest): tClientPutRequest {
        return (transId=req.transId, client=req.client, key=req.key, value=req.value, sender=this);
    }

    fun forwardedGetRequest(req: tClientGetRequest): tClientGetRequest {
        return (transId=req.transId, client=req.client, key=req.key, sender=this);
    }

    fun handleRequestVote(reply: tRequestVote) {
        // A handler for eRequestVote messages
        leader = default(Server);
        if (reply.term < currentTerm) {
            // reject a vote with a smaller term
            printLog(format("Reject vote with term {0} < currentTerm {1}", reply.term, currentTerm));
            send reply.candidate, eRequestVoteReply, (sender=this, term=currentTerm, voteGranted=false);
        } else {
            updateTermAndVote(reply.term);
            // if votedFor is null or the candidate it has voted for
            // and the log is up-to-date, grant the vote. Reject otherwise.
            if ((votedFor == default(Server) || votedFor == reply.candidate)
                    && logUpToDateCheck(reply.lastLogIndex, reply.lastLogTerm)) {
                printLog(format("Grant vote to {0} (prev. votedFor={1})", reply.candidate, votedFor));
                votedFor = reply.candidate;
                send reply.candidate, eRequestVoteReply, (sender=this, term=currentTerm, voteGranted=true);
            } else {
                printLog(format("Reject vote with votedFor={0} and logUpToDate={1}", votedFor, logUpToDateCheck(reply.lastLogIndex, reply.lastLogTerm)));
                send reply.candidate, eRequestVoteReply, (sender=this, term=currentTerm, voteGranted=false);
            }
        }
    }

    fun handleAppendEntries(resp: tAppendEntries) {
        // handler of leader heartbeat, log synchronization
        var myLastIndex: int;
        var myLastTerm: int;
        var ptr: int;
        var mismatchedTerm: int;
        var i: int;
        var j: int;
        printLog(format("Received AppendEntries({0}) from Leader({1}) with {2}; currentTerm={3}, log={4}", resp.entries, resp.leader, resp, currentTerm, logs));
        if (resp.term < currentTerm) {
            // if the heartbeat is from a leader with an outdated term, reject the heartbeat
            printLog(format("Reject AppendEntries with term {0} < currentTerm {1}", resp.term, currentTerm));
            send resp.leader, eAppendEntriesReply, (sender=this, term=currentTerm,
                                                    success=false, matchedIndex=-1);
        } else if (sizeof(logs) <= resp.prevLogIndex) {
            // If the log is very outdated, re-sync from the end of my log
            leader = resp.leader;
            printLog(format("prevLogIndex={0} is out of range of sizeof(log)={1}", resp.prevLogIndex, sizeof(logs)));
            send resp.leader, eAppendEntriesReply, (sender=this, term=currentTerm, success=false, matchedIndex=sizeof(logs));
        } else if (resp.prevLogIndex >= 0 && logs[resp.prevLogIndex].term != resp.prevLogTerm) {
            // Log consistency check failed here;
            // search for the first occurence of the mismatched
            // term and notify the leader.
            printLog(format("Log inconsistency at index {0}: log[i].term={1} v.s. prevTerm={2}", resp.prevLogIndex, logs[resp.prevLogIndex].term, resp.prevLogTerm));
            leader = resp.leader;
            ptr = resp.prevLogIndex;
            mismatchedTerm = logs[ptr].term;
            // find the first occurrence of the mismatched term
            // this includes the optimization described in the paper
            while (ptr > 0 && logs[ptr].term == mismatchedTerm) {
                ptr = ptr - 1;
            }
            printLog(format("requesting sync. log terms={0}, mismatchedTerm={1}, prevLogIndex={2}, ptr={3}", termsOfLog(), mismatchedTerm, resp.prevLogIndex, ptr));
            send resp.leader, eAppendEntriesReply, (sender=this, term=currentTerm,
                                                    success=false, matchedIndex=ptr);
        } else {
            // Check if any entries disagree; delete all entries after it.
            printLog(format("Sync log with log={0} at index {1}. Entries={2}", logs, resp.prevLogIndex + 1, resp.entries));
            leader = resp.leader;
            i = resp.prevLogIndex + 1;
            j = 0;
            printLog(format("Before sync: logs={0}", logs));
            // find the first mismatched entry
            while (i < sizeof(logs) && j < sizeof(resp.entries)) {
                if (logs[i].term != resp.entries[j].term) {
                    break;
                }
                i = i + 1;
                j = j + 1;
            }
            // delete all entries after the mismatched entry
            while (i < sizeof(logs)) {
                logs -= (lastLogIndex(logs));
            }
            printLog(format("Inconsistency removal: logs={0}", logs));
            // Append entries from the leader
            j = 0;
            while (j < sizeof(resp.entries)) {
                if (!(resp.entries[j] in logs)) {
                    logs += (resp.prevLogIndex + 1 + j, resp.entries[j]);
                } else {
                    assert resp.entries[j] == logs[resp.prevLogIndex + 1 + j], "Inconsistent log entry";
                }
                j = j + 1;
            }
            printLog(format("After sync: logs={0}, leaderCommit={1}, commitIndex={2}", logs, resp.leaderCommit, commitIndex));
            if (resp.leaderCommit > commitIndex) {
                if (resp.leaderCommit > lastLogIndex(logs)) {
                    commitIndex = lastLogIndex(logs);
                } else {
                    commitIndex = resp.leaderCommit;
                }
            }
            executeCommands();
            send resp.leader, eAppendEntriesReply, (sender=this, term=currentTerm, success=true, matchedIndex=lastLogIndex(logs));
        }
    }
    
    fun broadcastAppendEntries() {
        // broadcasting of leader heartbeats
        var target: Server;
        var i: int;
        var j: int;
        var prevIndex: int;
        var prevTerm: int;
        var entries: seq[tServerLog];
        assert this == leader, "Only leader can broadcast AppendEntries";
        // print format("Broadcasting AppendEntries with log {0}", logs);
        foreach (target in peers) {
            if (this != target) {
                entries = default(seq[tServerLog]);
                prevIndex = nextIndex[target] - 1;
                // get prevIndex and prevIndexTerm
                if (prevIndex >= 0 && prevIndex < sizeof(logs)) {
                    prevTerm = logs[prevIndex].term;
                } else {
                    prevTerm = 0;
                }
                if (nextIndex[target] < sizeof(logs)) {
                    // if the current lead has something to send (nextIndex within range of logs)
                    i = 0;
                    j = nextIndex[target];
                    // aggregate the logs to be synchronized
                    while (j < sizeof(logs)) {
                        entries += (i, logs[j]);
                        i = i + 1;
                        j = j + 1;
                    }
                    printLog(format("Entries for {0} is {1}", target, entries));
                    send target, eAppendEntries, (term=currentTerm,
                                                    leader=this,
                                                    prevLogIndex=prevIndex,
                                                    prevLogTerm=prevTerm,
                                                    entries=entries,
                                                    leaderCommit=commitIndex);
                } else {
                    // send empty heartbeat
                    send target, eAppendEntries, (term=currentTerm,
                                                    leader=this,
                                                    prevLogIndex=prevIndex,
                                                    prevLogTerm=prevTerm,
                                                    entries=entries,
                                                    leaderCommit=commitIndex);

                }
            }
        }
    }

    fun leaderCommits() {
        // leader committing log entries
        var execResult: ExecutionResult;
        // the next commit index
        var nextCommit: int;
        // the number of match indices that are greater than or equal to nextCommit
        var validMatchIndices: int;
        var i: int;
        var target: Server;
        assert this == leader, "Only leader can execute the log on its own";
        // iteratively search for that index
        nextCommit = lastLogIndex(logs);
        printLog(format("state: nextCommit={0} commitIndex={1} matchIndex={2}", nextCommit, commitIndex, matchIndex));
        // Find the largest nextCommit such that the majority of matchIndex is greater than or equal to nextCommit
        while (nextCommit > commitIndex) {
            // counting itself first
            validMatchIndices = 1;
            foreach (target in peers) {
                if (target == this) {
                    continue;
                }
                if (matchIndex[target] >= nextCommit) {
                    validMatchIndices = validMatchIndices + 1;
                }
            }
            // if the majority of matchIndex is greater than or equal to nextCommit
            // also the term is currentTerm in this entry
            if (validMatchIndices > clusterSize / 2 && logs[nextCommit].term == currentTerm) {
                commitIndex = nextCommit;
                break;
            }
            nextCommit = nextCommit - 1;
        }
        // commitIndex = nextCommit;
        printLog(format("leader commits decision: matchIndex={0} commitIndex={1} currentTerm={2} logs={3}", matchIndex, commitIndex, currentTerm, logs));
        // commit all the logs from lastApplied + 1 to commitIndex
        while (lastApplied < commitIndex) {
            lastApplied = lastApplied + 1;
            announce eEntryApplied, (logIndex=lastApplied, term=logs[lastApplied].term, key=logs[lastApplied].key, value=logs[lastApplied].value, transId=logs[lastApplied].transId, client=logs[lastApplied].client);
            execResult = executePut(kvStore, logs[lastApplied].key, logs[lastApplied].value);
            kvStore = execResult.newState;
            printLog(format("leader committed and processed (by {0}), log: {1}", this, logs[lastApplied]));
            // execute the command and send the result back to the client
            announce ePutResp, (client=logs[lastApplied].client, transId=logs[lastApplied].transId, key=logs[lastApplied].key, value=logs[lastApplied].value);
            send logs[lastApplied].client, eRaftPutResponse, (sender=this, client=logs[lastApplied].client,
                                                                transId=logs[lastApplied].transId, key=logs[lastApplied].key);
            // update the cache
            if (!(logs[lastApplied].client in keys(clientPutRequestCache))) {
                clientPutRequestCache[logs[lastApplied].client] = default(map[int, tRaftPutResponse]);
            }
            clientPutRequestCache[logs[lastApplied].client][logs[lastApplied].transId] = (sender=this, client=logs[lastApplied].client, transId=logs[lastApplied].transId, key=logs[lastApplied].key);
        }
    }

    fun checkClientRequestCache(client: Client, transactionId: TransId): bool {
        // if the result is already in the cache, send it back
        if (client in clientGetRequestCache && transactionId in keys(clientGetRequestCache[client])) {
            send client, eRaftGetResponse, clientGetRequestCache[client][transactionId];
            return true;
        }
        if (client in clientPutRequestCache && transactionId in keys(clientPutRequestCache[client])) {
            send client, eRaftPutResponse, clientPutRequestCache[client][transactionId];
            return true;
        }
        return false;
    }

    fun printLog(msg: string) {
        // return;
        // print format("{0}[{1}@{2}]: {3}", role, this, currentTerm, msg);
    }

    fun executeCommands() {
        var execResult: ExecutionResult;
        printLog(format("commitIndex={0} lastApplied={1}", commitIndex, lastApplied));
        // Execute the command from lastApplied + 1 to commitIndex
        while (lastApplied < commitIndex) {
            lastApplied = lastApplied + 1;
            announce eEntryApplied, (logIndex=lastApplied, term=logs[lastApplied].term, key=logs[lastApplied].key, value=logs[lastApplied].value, transId=logs[lastApplied].transId, client=logs[lastApplied].client);
            execResult = executePut(kvStore, logs[lastApplied].key, logs[lastApplied].value);
            kvStore = execResult.newState;
            // clientRequestCache[logs[lastApplied].client][logs[lastApplied].transId] = (client=logs[lastApplied].client, transId=logs[lastApplied].transId, result=execResult.result);
        }
    }

    fun logUpToDateCheck(lastIndex: LogIndex, lastTerm: TermId): bool {
        // Given \`lastIndex\` and \`lastTerm\` from the other node,
        // check if its logs is up-to-date.
        if (lastTerm > lastLogTerm(logs)) {
            // the lastTerm is greater than mine, then it is up-to-date
            return true;
        }
        if (lastTerm < lastLogTerm(logs)) {
            // the lastTerm is smaller than mine, then it is not up-to-date
            return false;
        }
        // the lastTerm is the same, then whichever's log is longer is up-to-date
        return lastIndex >= lastLogIndex(logs);
    }

    fun notifyViewLog() {
        logNotifyTimestamp = logNotifyTimestamp + 1;
        send viewServer, eNotifyLog, (timestamp=logNotifyTimestamp, server=this, log=logs);
    }

    fun becomeFollower(term: int) {
        // cancelTimer(electionTimer);
        updateTermAndVote(term);
        goto Follower;
    }

    fun updateTermAndVote(term: int) {
        if (term > currentTerm) {
            // if the term is changed, then reset the votedFor since
            // it has not voted in the new term.
            votedFor = default(Server);
        }
        // the only case is currentTerm == term
        currentTerm = term;
    }

    fun termsOfLog(): seq[int] {
        var terms: seq[int];
        var i: int;
        terms = default(seq[int]);
        i = 0;
        while (i < sizeof(logs)) {
            terms += (sizeof(terms), logs[i].term);
            i = i + 1;
        }
        return terms;
    }

    fun reset() {
        // crash a node, reset volatile states
        // and become a follower (just as it starts up)
        commitIndex = -1;
        lastApplied = -1;
        nextIndex = fillMap(this, nextIndex, peers, 0);
        matchIndex = fillMap(this, matchIndex, peers, -1);
        goto Follower;
    }
}`,rb=`/******************************************************************
* A Timer machine that timeouts non-deterministically.
*******************************************************************/

event eStartTimer;
event eElectionTimeout;
event eHeartbeatTimeout;
event eCancelTimer;
event eTick;

machine Timer {
    var holder: machine;
    var timeoutEvent: event;

    start state Init {
        entry (setup: (user: machine, timeoutEvent: event)) {
            holder = setup.user;
            timeoutEvent = setup.timeoutEvent;
            goto TimerIdle;
        }
        ignore eCancelTimer, eTick, eStartTimer;
    }

    state TimerIdle {
        on eStartTimer do {
            goto TimerTick;
        }
        on eShutdown goto TimerShutdown;
        ignore eCancelTimer, eTick;
    }

    state TimerTick {
        entry {
            checkTick();
        }
        on eTick do {
            checkTick();
        }
        on eShutdown goto TimerShutdown;
        on eCancelTimer goto TimerIdle;
        ignore eStartTimer;
    }

    state TimerShutdown {
        ignore eStartTimer, eCancelTimer, eTick;
    }

    fun checkTick() {
        if ($) {
            // non-deterministic timeout
            send holder, timeoutEvent;
            goto TimerIdle;
        } else {
            send this, eTick;
        }
    }
}`,ib=`// Some utility functions
fun startTimer(timer: Timer) {
    send timer, eStartTimer;
}

fun cancelTimer(timer: Timer) {
    send timer, eCancelTimer;
}

fun restartTimer(timer: Timer) {
    cancelTimer(timer);
    startTimer(timer);
}

fun lastLogIndex(log: seq[tServerLog]): LogIndex {
    return sizeof(log) - 1;
}

fun getLogTerm(log: seq[tServerLog], index: int): int {
    // get the term of the log entry at index.
    if (index < 0 || index >= sizeof(log)) {
        return 0;
    } else {
        return log[index].term;
    }
}

fun lastLogTerm(log: seq[tServerLog]): int {
    if (sizeof(log) == 0) {
        return 0;
    } else {
        return log[sizeof(log) - 1].term;
    }
}

fun Max(a: int, b: int): int {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}

fun Min(a: int, b: int): int {
    if (a < b) {
        return a;
    } else {
        return b;
    }
}

fun fillMap(self: machine, m: map[Server, int], servers: set[Server], value: int): map[Server, int] {
    var server: Server;
    foreach(server in servers) {
        if (server != self) {
            m[server] = value;
        }
    }
    return m;
}

fun broadcastRequest(self: Server, peers: set[Server], e: event, payload: any) {
    // broadcast a message to nodes except for itself.
    var peer: Server;
    foreach(peer in peers) {
        if (peer != self) {
            send peer, e, payload;
        }
    }
}
`,sb=`/******************************************************************
* A View service that manages the view of the cluster.
* It is responsible for maintaining roles of, injecting failures to
* and shutting down the nodes.
*******************************************************************/

// Shutdown message will be sent to the cluster when
// client notifies the view service that it has finished all requests AND got all responses
event eShutdown;
// A node can notify the view service about its log entires
// This is used for choosing servers that should receive an election timeout
type tTS = int;
event eNotifyLog: (timestamp: tTS, server: Server, log: seq[tServerLog]);

machine View {
    // the cluster nodes
    var servers: set[Server];
    // the probability that a node got an election timeout [0, 100]
    // this is used to model network failure (i.e. a follower does not receive heartbeat)
    var timeoutRate: int;
    // the probability that a node crashes [0, 100]
    var crashRate: int;
    // the total number failures that can be injected
    var numFailures: int;
    // the timer for the view service to perform an action
    var triggerTimer: Timer;
    // the set of clients that have finished all requests
    var clientsDone: set[machine];
    // the number of clients
    var numClients: int;

    // the set of followers, leaders and candidates
    var followers: set[Server];
    var leaders: set[Server];
    var candidates: set[Server];
    // the set of servers that have not sent a \`eRequestVote\` message
    // this set will be exahusted when the view service
    // cannot detect any leader in the cluster
    var requestVotePendingSet: set[Server];
    // the logs of nodes
    var serverLogs: map[Server, seq[tServerLog]];
    // the last seen log of each node; prevent update the entry using older logs
    var lastSeenLogs: map[Server, int];
    var candidateRoundMap: map[Server, int];
    // number of rounds of actions where the view service cannot detect any leader
    var noLeaderRounds: int;

    start state Init {
        entry (setup: (numServers: int, numClients: int, timeoutRate: int, crashRate: int, numFailures: int)) {
            var i: int;
            var server: Server;
            timeoutRate = setup.timeoutRate;
            numClients = setup.numClients;
            crashRate = setup.crashRate;
            numFailures = setup.numFailures;
            followers = default(set[Server]);
            // at a moment, there can be multiple leaders, e.g. partitioned network
            leaders = default(set[Server]);
            candidates = default(set[Server]);
            requestVotePendingSet = default(set[Server]);
            serverLogs = default(map[Server, seq[tServerLog]]);
            lastSeenLogs = default(map[Server, int]);
            candidateRoundMap = default(map[Server, int]);
            noLeaderRounds = 0;

            while (i < setup.numServers) {
                server = new Server();
                servers += (server);
                serverLogs += (server, default(seq[tServerLog]));
                i = i + 1;
            }
            i = 0;
            foreach (server in servers) {
                send server, eServerInit, (myId=i, cluster=servers, viewServer=this);
                followers += (server);
                i = i + 1;
            }
            triggerTimer = new Timer((user=this, timeoutEvent=eHeartbeatTimeout));
            clientsDone = default(set[machine]);
            i = 0;
            while (i < numClients) {
                new Client((viewService=this, servers=servers, requests=randomWorkload(3)));
                i = i + 1;
            } 
            goto Monitoring;
        }
    }

    state Monitoring {
        entry {
            var server: Server;
            startTimer(triggerTimer);
            server = choose(servers);
            candidates += (server);
            send server, eElectionTimeout;
        }

        on eNotifyLog do (payload: (timestamp:int, server: Server, log: seq[tServerLog])) {
            // only track the most up-to-date logs
            if (!(payload.server in keys(lastSeenLogs)) || lastSeenLogs[payload.server] < payload.timestamp) {
                lastSeenLogs[payload.server] = payload.timestamp;
                serverLogs[payload.server] = payload.log;
            }
        }

        on eViewChangedLeader do (server: Server) {
            // server change its role to a leader
            noLeaderRounds = 0;
            leaders += (server);
            followers -= (server);
            candidates -= (server);
        }

        on eViewChangedFollower do (server: Server) {
            // server change its role to a follower
            followers += (server);
            candidates -= (server);
            leaders -= (server);
        }

        on eViewChangedCandidate do (server: Server) {
            // server change its role to a candidate
            candidates += (server);
            followers -= (server);
            leaders -= (server);
        }

        on eHeartbeatTimeout do {
            var server: Server;
            // print format("Current view: leaders={0} followers={1} candidates={2}", leaders, followers, candidates);
            if (sizeof(leaders) == 0) {
                if (noLeaderRounds % 25 == 0 && sizeof(requestVotePendingSet) > 0) {
                    // non-deterministically choose a server to trigger an election
                    // either the current most up-to-date server or a random server
                    if ($) {
                        server = mostUpToDateServer(requestVotePendingSet);
                    } else {
                        server = choose(requestVotePendingSet);
                    }
                    requestVotePendingSet -= (server);
                    // print format("NoLeader rounds exceeded, trigger election on {0}", server);
                    send server, eElectionTimeout;
                    candidates += (server);
                    followers -= (server);
                    noLeaderRounds = 0;
                }
                if (sizeof(requestVotePendingSet) == 0) {
                    requestVotePendingSet = servers;
                }
                noLeaderRounds = noLeaderRounds + 1;
            } else {
                foreach (server in servers) {
                    send server, eHeartbeatTimeout;
                }
                foreach (server in followers) {
                    if (choose(100) < timeoutRate && numFailures > 0) {
                        // print format("Failure Injection: timeout a follower {0}", server);
                        send server, eElectionTimeout;
                        numFailures = numFailures - 1;
                        startTimer(triggerTimer);
                        return;
                    }
                }
                foreach (server in leaders) {
                    if (choose(100) < crashRate && numFailures > 0) {
                        // print format("Failure Injection: crash a leader {0}", server);
                        send server, eReset;
                        numFailures = numFailures - 1;
                        startTimer(triggerTimer);
                        return;
                    }
                }
                foreach (server in candidates) {
                    if (choose(100) < crashRate && numFailures > 0) {
                        // print format("Failure Injection: crash a candidate {0}", server);
                        send server, eReset;
                        numFailures = numFailures - 1;
                        startTimer(triggerTimer);
                        return;
                    }
                }
            }
            startTimer(triggerTimer);
        }

        on eClientFinished do (client: machine) {
            var i: int;
            clientsDone += (client);
            if (sizeof(clientsDone) == numClients) {
                i = 0;
                while (i < sizeof(servers)) {
                    send servers[i], eShutdown;
                    i = i + 1;
                }
                goto ViewServiceEnd;   
            }
        }
    }

    state ViewServiceEnd {
        entry {
            send triggerTimer, eShutdown;
        }

        ignore eClientFinished, eHeartbeatTimeout, eViewChangedCandidate, eNotifyLog, eViewChangedLeader, eViewChangedFollower;
    }

    fun mostUpToDateServer(choices: set[Server]): Server {
        // get the server with the most up-to-date logs (in the view of the view service)
        // it might not be the real most up-to-date server in the cluster because of the message delay
        var server: Server;
        var candidate: Server;
        var term: int;
        var length: int;
        term = 0;
        length = 0;
        candidate = default(Server);
        // print format("Choose candidate: server |-> logs: {0}", serverLogs);
        foreach (server in servers) {
            if (candidate == default(Server)) {
                candidate = server;
                term = lastLogTerm(serverLogs[server]);
                length = sizeof(serverLogs[server]);
            } else {
                if (sizeof(serverLogs[server]) > 0) {
                    if (term < lastLogTerm(serverLogs[server])) {
                        term = lastLogTerm(serverLogs[server]);
                        length = sizeof(serverLogs[server]);
                        candidate = server;
                    } else if (term == lastLogTerm(serverLogs[server])) {
                        if (length < sizeof(serverLogs[server])) {
                            length = sizeof(serverLogs[server]);
                            candidate = server;
                        }
                    }
                }
            }
        }
        return candidate;
    }
}
`,ob=`event eRaftConfig: (quorumSize: int);

fun setUpCluster(numServers: int, numClients: int, timeoutRate: int, crashRate: int, numFailures: int): View {
    announce eRaftConfig, (quorumSize=numServers / 2 + 1,);
    return new View((numServers=numServers, numClients=numClients,
                                timeoutRate=timeoutRate, crashRate=crashRate, numFailures=numFailures));
}

fun randomWorkload(numCmd: int): seq[Command] {
    var cmds: seq[Command];
    var puts: set[int];
    var i: int;
    var key: int;
    assert numCmd <= 100, "Too many commands!";
    cmds = default(seq[Command]);
    puts = default(set[int]);
    i = 0;
    while (i < numCmd) {
        // choose an existing key or a new key
        // non-deterministically
        if (sizeof(puts) > 0 && $) {
            key = choose(puts);
        } else {
            key = choose(10);
            puts += (key);
        }
        if (sizeof(puts) == 0 || $) {
            // PUT
            cmds += (i, (op=PUT, key=key, value=choose(1024)));
        } else {
            // GET
            cmds += (i, (op=GET, key=key, value=-1));
        }
        i = i + 1;
    }
    return cmds;
}

machine OneClientOneServerReliable {
    start state Init {
        entry {
            var view: View;
            view = setUpCluster(1, 1, 0, 0, 0);
        }
    }
}

machine OneClientFiveServersReliable {
    start state Init {
        entry {
            var view: View;
            view = setUpCluster(5, 1, 0, 0, 0);
        }
    }
}

machine OneClientFiveServersUnreliable {
    start state Init {
        entry {
            var view: View;
            view = setUpCluster(5, 1, 5, 5, 100);
        }
    }
}

machine OneClientThreeServersReliable {
    start state Init {
        entry {
            var view: View;
            view = setUpCluster(3, 1, 0, 0, 0);
        }
    }
}

machine OneClientThreeServersUnreliable {
    start state Init {
        entry {
            var view: View;
            view = setUpCluster(3, 1, 5, 5, 100);
        }
    }
}

machine TwoClientsThreeServersReliable {
    start state Init {
        entry {
            var view: View;
            view = setUpCluster(3, 2, 0, 0, 0);
        }
    }
}

machine TwoClientsThreeServersUnreliable {
    start state Init {
        entry {
            var view: View;
            view = setUpCluster(3, 2, 5, 5, 100);
        }
    }
}

machine ThreeClientsOneServerReliable {
    start state Init {
        entry {
            var view: View;
            view = setUpCluster(1, 3, 0, 0, 0);
        }
    }
}
`,lb=`module ServingTests = { OneClientOneServerReliable,
                        OneClientThreeServersReliable,
                        OneClientThreeServersUnreliable,
                        OneClientFiveServersReliable,
                        OneClientFiveServersUnreliable,
                        TwoClientsThreeServersReliable,
                        TwoClientsThreeServersUnreliable,
                        ThreeClientsOneServerReliable};

test oneClientOneServerReliable [main=OneClientOneServerReliable]:
  assert SafetyOneLeader, SafetyLeaderCompleteness, SafetyStateMachine, SafetyLogMatching, LivenessClientsDone, LivenessProgress, SafetySynchronization in
  (union Server, Timer, Client, View, ServingTests);

test oneClientThreeServersReliable [main=OneClientThreeServersReliable]:
  assert SafetyOneLeader, SafetyLeaderCompleteness, SafetyStateMachine, SafetyLogMatching, LivenessClientsDone, LivenessProgress, SafetySynchronization in
  (union Server, Timer, Client, View, ServingTests);

test oneClientThreeServersUnreliable [main=OneClientThreeServersUnreliable]:
  assert SafetyOneLeader, SafetyLeaderCompleteness, SafetyStateMachine, SafetyLogMatching, LivenessClientsDone, LivenessProgress, SafetySynchronization in
  (union Server, Timer, Client, View, ServingTests);

test oneClientFiveServersReliable [main=OneClientFiveServersReliable]:
  assert SafetyOneLeader, SafetyLeaderCompleteness, SafetyStateMachine, SafetyLogMatching, LivenessClientsDone, LivenessProgress, SafetySynchronization in
  (union Server, Timer, Client, View, ServingTests);

test oneClientFiveServersUnreliable [main=OneClientFiveServersUnreliable]:
  assert SafetyOneLeader, SafetyLeaderCompleteness, SafetyStateMachine, SafetyLogMatching, LivenessClientsDone, LivenessProgress, SafetySynchronization in
  (union Server, Timer, Client, View, ServingTests);

test threeClientsOneServerReliable [main=ThreeClientsOneServerReliable]:
  assert SafetyOneLeader, SafetyLeaderCompleteness, SafetyStateMachine, SafetyLogMatching, LivenessClientsDone, LivenessProgress, SafetySynchronization in
  (union Server, Timer, Client, View, ServingTests);

test twoClientsThreeServersReliable [main=TwoClientsThreeServersReliable]:
  assert SafetyOneLeader, SafetyLeaderCompleteness, SafetyStateMachine, SafetyLogMatching, LivenessClientsDone, LivenessProgress, SafetySynchronization in
  (union Server, Timer, Client, View, ServingTests);

test twoClientsThreeServersUnreliable [main=TwoClientsThreeServersUnreliable]:
  assert SafetyOneLeader, SafetyLeaderCompleteness, SafetyStateMachine, SafetyLogMatching, LivenessClientsDone, LivenessProgress, SafetySynchronization in
  (union Server, Timer, Client, View, ServingTests);`,ab=`/*
The failure injector machine randomly selects a replica machine and enqueues the special event "halt".
*/
event eDelayNodeFailure;
// event: event sent by the failure injector to shutdown a node
event eShutDown: machine;

machine FailureInjector {
  var nFailures: int;
  var nodes: set[machine];

  start state Init {
    entry (config: (nodes: set[machine], nFailures: int)) {
      nFailures = config.nFailures;
      nodes = config.nodes;
      assert nFailures < sizeof(nodes);
      goto FailOneNode;
    }
  }

  state FailOneNode {
    entry {
      var fail: machine;

      if(nFailures == 0)
        raise halt; // done with all failures
      else
      {
        if($)
        {
          fail = choose(nodes);
          send fail, eShutDown, fail;
          nodes -= (fail);
          nFailures = nFailures - 1;
        }
        else {
          send this, eDelayNodeFailure;
        }
      }
    }

    on eDelayNodeFailure goto FailOneNode;
  }
}

// function to create the failure injection
fun CreateFailureInjector(config: (nodes: set[machine], nFailures: int)) {
  new FailureInjector(config);
}

// failure injector module
module FailureInjector = { FailureInjector };`,cb=`// unreliable send operation that drops messages on the ether nondeterministically
fun UnReliableSend(target: machine, message: event, payload: any) {
  // nondeterministically drop messages
  // $: choose()
  if($) send target, message, payload;
}

// unrelialbe broadcast function
fun UnReliableBroadCast(ms: set[machine], ev: event, payload: any) {
  var i: int;
  while (i < sizeof(ms)) {
    UnReliableSend(ms[i], ev, payload);
    i = i + 1;
  }
}

// relialbe broadcast function
fun ReliableBroadCast(ms: set[machine], ev: event, payload: any) {
  var i: int;
  while (i < sizeof(ms)) {
    send ms[i], ev, payload;
    i = i + 1;
  }
}`,ub=`/*****************************************************************************************
The timer state machine models the non-deterministic behavior of an OS timer
******************************************************************************************/

/************************************************
Events used to interact with the timer machine
************************************************/
event eStartTimer;
event eCancelTimer;
event eTimeOut;
event eDelayedTimeOut;

machine Timer {
  // user/client of the timer
  var client: machine;

  start state Init {
    entry (_client : machine) {
      client = _client;
      goto WaitForTimerRequests;
    }
  }

  state WaitForTimerRequests {
    on eStartTimer goto TimerStarted;

    ignore eCancelTimer, eDelayedTimeOut;
  }

  state TimerStarted {
    entry {
      // Only fire the timer with a chance of 1/10 to avoid livelocks
      if(choose(10) == 0) {
        send client, eTimeOut;
        goto WaitForTimerRequests;
      } else {
        send this, eDelayedTimeOut;
      }
    }

    on eDelayedTimeOut goto TimerStarted;
    on eCancelTimer goto WaitForTimerRequests;
    defer eStartTimer;
  }
}

/************************************************
Functions or API's to interact with the OS Timer
*************************************************/
// create timer
fun CreateTimer(client: machine) : Timer {
  return new Timer(client);
}

// start timer
fun StartTimer(timer: Timer) {
  send timer, eStartTimer;
}

// cancel timer
fun CancelTimer(timer: Timer) {
  send timer, eCancelTimer;
}
`,hb=`/* Create the timer module which consists of only the timer machine */
module Timer = { Timer };`,fb=`/*
  Welcome to the P playground!

  P models a system as state machines
  that talk by sending events. The
  checker runs the model many times,
  trying different event orderings,
  and reports any schedule that breaks
  an assertion.

  Here two clients increment a shared
  counter with a read, then a write.
  Press Check to find the interleaving
  that loses an update, then step
  through the trace.

  To fix it, replace the read/write
  pair with one eIncrement event that
  the Counter handles atomically.
*/

event eRead: Client;          // ask the counter for its value
event eValue: int;            // the counter's reply
event eWrite: int;            // overwrite the counter
event eDone;                  // a client finished
event eGet: Main;             // the test asks for the final value

machine Counter {
  var value: int;
  start state Serving {
    on eRead do (c: Client) { send c, eValue, value; }
    on eWrite do (v: int) { value = v; }
    on eGet do (m: Main) { send m, eValue, value; }
  }
}

machine Client {
  start state Run {
    entry (input: (counter: Counter, owner: Main)) {
      var v: int;
      send input.counter, eRead, this;
      receive { case eValue: (x: int) { v = x; } }
      send input.counter, eWrite, v + 1;
      send input.owner, eDone;
    }
  }
}

machine Main {
  var counter: Counter;
  var done: int;
  start state Init {
    entry {
      counter = new Counter();
      new Client((counter = counter, owner = this));
      new Client((counter = counter, owner = this));
    }
    on eDone do {
      done = done + 1;
      if (done == 2) {
        send counter, eGet, this;
        goto Verify;
      }
    }
  }
  state Verify {
    on eValue do (final: int) {
      assert final == 2, format("expected 2 increments, counter is {0}", final);
    }
  }
}

test tcTwoClients [main=Main]: { Main, Counter, Client };
`,db=Object.assign({"../../design/time/RetryBudget/PSpec/Budget.p":oC,"../../design/time/RetryBudget/PSrc/RetryClient.p":lC,"../../design/time/RetryBudget/PSrc/Time.p":aC,"../../design/time/RetryBudget/PSrc/Timer.p":cC,"../../design/time/RetryBudget/PTst/Test.p":uC,"../../design/time/RetryBudget/PTst/latency.toml":hC,"../../tests/upstream/Tutorial/1_ClientServer/PSpec/BankBalanceCorrect.p":fC,"../../tests/upstream/Tutorial/1_ClientServer/PSrc/AbstractBankServer.p":dC,"../../tests/upstream/Tutorial/1_ClientServer/PSrc/Client.p":pC,"../../tests/upstream/Tutorial/1_ClientServer/PSrc/ClientServerModules.p":mC,"../../tests/upstream/Tutorial/1_ClientServer/PSrc/Server.p":gC,"../../tests/upstream/Tutorial/1_ClientServer/PTst/TestDriver.p":vC,"../../tests/upstream/Tutorial/1_ClientServer/PTst/Testscript.p":yC,"../../tests/upstream/Tutorial/2_TwoPhaseCommit/PSpec/Atomicity.p":wC,"../../tests/upstream/Tutorial/2_TwoPhaseCommit/PSrc/Coordinator.p":SC,"../../tests/upstream/Tutorial/2_TwoPhaseCommit/PSrc/Participant.p":kC,"../../tests/upstream/Tutorial/2_TwoPhaseCommit/PSrc/TwoPhaseCommitModules.p":xC,"../../tests/upstream/Tutorial/2_TwoPhaseCommit/PTst/Client.p":CC,"../../tests/upstream/Tutorial/2_TwoPhaseCommit/PTst/TestDriver.p":bC,"../../tests/upstream/Tutorial/2_TwoPhaseCommit/PTst/TestScripts.p":TC,"../../tests/upstream/Tutorial/3_EspressoMachine/PSpec/Safety.p":RC,"../../tests/upstream/Tutorial/3_EspressoMachine/PSrc/CoffeeMaker.p":MC,"../../tests/upstream/Tutorial/3_EspressoMachine/PSrc/CoffeeMakerControlPanel.p":IC,"../../tests/upstream/Tutorial/3_EspressoMachine/PSrc/EspressoMachineModules.p":AC,"../../tests/upstream/Tutorial/3_EspressoMachine/PTst/TestDrivers.p":PC,"../../tests/upstream/Tutorial/3_EspressoMachine/PTst/TestScripts.p":EC,"../../tests/upstream/Tutorial/3_EspressoMachine/PTst/Users.p":LC,"../../tests/upstream/Tutorial/4_FailureDetector/PSpec/ReliableFailureDetector.p":DC,"../../tests/upstream/Tutorial/4_FailureDetector/PSrc/Client.p":OC,"../../tests/upstream/Tutorial/4_FailureDetector/PSrc/FailureDetector.p":NC,"../../tests/upstream/Tutorial/4_FailureDetector/PSrc/FailureDetectorModules.p":BC,"../../tests/upstream/Tutorial/4_FailureDetector/PSrc/Node.p":_C,"../../tests/upstream/Tutorial/4_FailureDetector/PTst/TestDriver.p":qC,"../../tests/upstream/Tutorial/4_FailureDetector/PTst/TestScript.p":WC,"../../tests/upstream/Tutorial/5_Paxos/PSpec/common.p":FC,"../../tests/upstream/Tutorial/5_Paxos/PSpec/spec.p":zC,"../../tests/upstream/Tutorial/5_Paxos/PSrc/acceptor.p":jC,"../../tests/upstream/Tutorial/5_Paxos/PSrc/learner.p":VC,"../../tests/upstream/Tutorial/5_Paxos/PSrc/proposer.p":HC,"../../tests/upstream/Tutorial/5_Paxos/PTst/test.p":UC,"../../tests/upstream/Tutorial/6_Raft/PSpec/LivenessClientsDone.p":$C,"../../tests/upstream/Tutorial/6_Raft/PSpec/LivenessProgress.p":GC,"../../tests/upstream/Tutorial/6_Raft/PSpec/SafetyLeaderCompleteness.p":KC,"../../tests/upstream/Tutorial/6_Raft/PSpec/SafetyLogMatching.p":QC,"../../tests/upstream/Tutorial/6_Raft/PSpec/SafetyOneLeader.p":YC,"../../tests/upstream/Tutorial/6_Raft/PSpec/SafetyStateMachine.p":XC,"../../tests/upstream/Tutorial/6_Raft/PSpec/SafetySynchronization.p":JC,"../../tests/upstream/Tutorial/6_Raft/PSrc/Application.p":ZC,"../../tests/upstream/Tutorial/6_Raft/PSrc/Client.p":eb,"../../tests/upstream/Tutorial/6_Raft/PSrc/RaftModules.p":tb,"../../tests/upstream/Tutorial/6_Raft/PSrc/Server.p":nb,"../../tests/upstream/Tutorial/6_Raft/PSrc/Timer.p":rb,"../../tests/upstream/Tutorial/6_Raft/PSrc/Utils.p":ib,"../../tests/upstream/Tutorial/6_Raft/PSrc/View.p":sb,"../../tests/upstream/Tutorial/6_Raft/PTst/RaftTests.p":ob,"../../tests/upstream/Tutorial/6_Raft/PTst/Testscript.p":lb,"../../tests/upstream/Tutorial/Common/FailureInjector/PSrc/FailureInjector.p":ab,"../../tests/upstream/Tutorial/Common/FailureInjector/PSrc/NetworkFunctions.p":cb,"../../tests/upstream/Tutorial/Common/Timer/PSrc/Timer.p":ub,"../../tests/upstream/Tutorial/Common/Timer/PSrc/TimerModules.p":hb,"../demos/hello/LostUpdate.p":fb}),rn="../../tests/upstream/Tutorial/",xr=[{id:"lost-update",title:"Hello P: a lost update",blurb:"Two clients increment a shared counter with a read and then a write. Check it to find the interleaving that loses an update, then step through the trace.",roots:[{from:"../demos/hello/"}],open:"LostUpdate.p",testCase:"tcTwoClients",schedules:100},{id:"client-server",title:"Bank client/server",blurb:"Clients withdraw money from a bank server backed by a database. A safety spec checks balances never drop below 10; a liveness spec checks every request gets a reply. tcAbstractServer uses a deliberately buggy abstract bank.",roots:[{from:`${rn}1_ClientServer/`}],open:"PSrc/Server.p",testCase:"tcAbstractServer",schedules:1e3},{id:"two-phase-commit",title:"Two-phase commit",blurb:"A coordinator runs two-phase commit over participants, with timers and an optional failure injector. The Atomicity spec checks every participant agrees on each transaction.",roots:[{from:`${rn}2_TwoPhaseCommit/`},{from:`${rn}Common/Timer/`,to:"Common/Timer/"},{from:`${rn}Common/FailureInjector/`,to:"Common/FailureInjector/"}],open:"PSrc/Coordinator.p",testCase:"tcMultipleClientsWithFailure",schedules:1e3},{id:"espresso",title:"Espresso machine",blurb:"A coffee maker and its control panel, driven by sane and 'crazy' users pressing buttons in any order. The spec checks the machine never brews in an invalid state.",roots:[{from:`${rn}3_EspressoMachine/`}],open:"PSrc/CoffeeMakerControlPanel.p",testCase:"tcCrazyUserUsingCoffeeMachine",schedules:1e3},{id:"failure-detector",title:"Failure detector",blurb:"A heartbeat-based failure detector with nodes that can crash. The spec checks that only crashed nodes are ever reported as failed.",roots:[{from:`${rn}4_FailureDetector/`},{from:`${rn}Common/Timer/`,to:"Common/Timer/"},{from:`${rn}Common/FailureInjector/`,to:"Common/FailureInjector/"}],open:"PSrc/FailureDetector.p",testCase:"tcTest_FailureDetector",schedules:1e3},{id:"paxos",title:"Single-decree Paxos",blurb:"Proposers, acceptors and learners. The spec checks at most one value is ever chosen.",roots:[{from:`${rn}5_Paxos/`}],open:"PSrc/proposer.p",testCase:"testBasicPaxos3on3",schedules:1e3},{id:"raft",title:"Raft",blurb:"Leader election and log replication with unreliable networks, checked against safety (one leader per term, log matching, …) and liveness specs. Larger: expect a few seconds per 100 schedules.",roots:[{from:`${rn}6_Raft/`}],open:"PSrc/Server.p",testCase:"oneClientThreeServersUnreliable",schedules:100},{id:"retry-budget",title:"Time: retry budget",blurb:"Uses the prs time extension: a client retries with timeouts and backoff, and a spec asserts every request resolves within a 1 s budget. Set timeoutAfter = ms(250) in PSrc/RetryClient.p to fix it.",roots:[{from:"../../design/time/RetryBudget/"}],open:"PSrc/RetryClient.p",testCase:"tcRetry",schedules:1e3,extraArgs:"--latency PTst/latency.toml --timed"}],om=["PSrc/","PSpec/","PTst/","Common/"];function lm(t){const e=n=>{const r=om.findIndex(i=>n.startsWith(i));return r<0?n.includes("/")?om.length:-1:r};return[...t].sort((n,r)=>e(n)-e(r)||n.localeCompare(r))}function pb(t){const e={};for(const n of t.roots)for(const[r,i]of Object.entries(db))r.startsWith(n.from)&&(e[(n.to??"")+r.slice(n.from.length)]=i);return e}let ph=[],ay=[];(()=>{let t="lc,34,7n,7,7b,19,,,,2,,2,,,20,b,1c,l,g,,2t,7,2,6,2,2,,4,z,,u,r,2j,b,1m,9,9,,o,4,,9,,3,,5,17,3,1n,9,16,o,,x,1i,3,,i,,7,a,2,t,3,1k,,,7,2,2,2,3,9,,a,2,q,,2,3,1k,,,5,4,2,2,3,3,,u,2,3,,b,3,1k,,,8,,3,,3,k,2,m,6,,3,1k,,,7,2,2,2,3,7,3,a,2,u,,1n,5,3,3,,4,9,,14,5,1j,,,7,,3,,4,7,2,b,2,t,3,1k,,,7,,3,,4,7,2,b,2,f,,c,4,1j,2,,7,,3,,4,9,,a,2,t,3,1y,,4,6,,,,8,i,2,1p,,,8,c,8,2q,,,a,b,7,21,2,r,,,,,,4,2,1d,k,,2,5,b,,10,9,,2u,b,,6,n,4,4,3,g,4,d,,,3,6,,f,,jj,3,qa,4,s,3,t,2,u,2,1s,w,9,,19,3,,,39,2,y,,3a,c,4,c,63,5,1l,a,,,,,2,o,2,,1c,1a,2,c,k,5,1b,h,12,9,c,3,u,d,1k,e,1c,k,48,3,,l,4,,6,,2,3,5i,1s,ek,,5f,x,2da,3,3x,,2o,w,fe,6,2x,2,n9w,4,,a,w,2,28,2,7k,,3,,4,,n,5,4,,2b,2,1e,i,q,i,d,,12,8,p,d,18,4,1b,e,10,,1v,e,c,,8,2,1a,,1f,,,3,2,2,5,2,,,15,5,5,2,6k,8,,2,fn4,,kh,g,g,g,a6,2,gt,,6a,,45,5,1ae,3,,2,5,4,14,3,4,,4l,2,fx,4,1t,5,8t,2,25,6,1y,b,1d,4,3e,3,1h,f,15,,2,2,a,4,19,b,7,,1p,3,10,e,g,2,18,,c,3,1c,e,8,4,,2,2k,c,6,,2,,4d,c,l,4,1j,2,,7,2,2,2,3,9,,a,2,2,7,3,5,1v,9,,,2,,,4,,5,,,e,2,2a,i,n,,29,k,6j,7,2,9,r,2,2a,h,2y,d,2t,3,2,a,74,f,6t,6,,2,2,4,,,,2,3x,7,2,7,3,,s,a,14,7,,4,8,,9,b,1a,g,5i,8,5j,8,,8,2a,m,,e,3e,6,3,,,2,,7,,,1u,5,,2,,5,9n,4,9,2,,,1c,7,3,5,n,,44l,,6,f,8ug,i,1xc,5,1n,7,t4,,,1j,7,4,29,,b,2,f57,2,3mp,1a,2,n,f2,5,3,6,8,8,2,7,u,4,44,3,1iz,1j,4,1e,8,,e,,m,5,,f,11s,7,,h,2,7,,2,,5,2s,,4g,7,af,,1p,4,e4,4,72,2,6r,,2,,7,2,5,,d6,7,31,7,240,5".split(",").map(e=>e?parseInt(e,36):1);for(let e=0,n=0;e<t.length;e++)(e%2?ay:ph).push(n=n+t[e])})();function mb(t){if(t<768)return!1;for(let e=0,n=ph.length;;){let r=e+n>>1;if(t<ph[r])n=r;else if(t>=ay[r])e=r+1;else return!0;if(e==n)return!1}}function am(t){return t>=127462&&t<=127487}const cm=8205;function gb(t,e,n=!0,r=!0){return(n?cy:vb)(t,e,r)}function cy(t,e,n){if(e==t.length)return e;e&&uy(t.charCodeAt(e))&&hy(t.charCodeAt(e-1))&&e--;let r=Hc(t,e);for(e+=um(r);e<t.length;){let i=Hc(t,e);if(r==cm||i==cm||n&&mb(i))e+=um(i),r=i;else if(am(i)){let s=0,o=e-2;for(;o>=0&&am(Hc(t,o));)s++,o-=2;if(s%2==0)break;e+=2}else break}return e}function vb(t,e,n){for(;e>1;){let r=cy(t,e-2,n);if(r<e)return r;e--}return 0}function Hc(t,e){let n=t.charCodeAt(e);if(!hy(n)||e+1==t.length)return n;let r=t.charCodeAt(e+1);return uy(r)?(n-55296<<10)+(r-56320)+65536:n}function uy(t){return t>=56320&&t<57344}function hy(t){return t>=55296&&t<56320}function um(t){return t<65536?1:2}class te{lineAt(e){if(e<0||e>this.length)throw new RangeError(`Invalid position ${e} in document of length ${this.length}`);return this.lineInner(e,!1,1,0)}line(e){if(e<1||e>this.lines)throw new RangeError(`Invalid line number ${e} in ${this.lines}-line document`);return this.lineInner(e,!0,1,0)}replace(e,n,r){[e,n]=ji(this,e,n);let i=[];return this.decompose(0,e,i,2),r.length&&r.decompose(0,r.length,i,3),this.decompose(n,this.length,i,1),hn.from(i,this.length-(n-e)+r.length)}append(e){return this.replace(this.length,this.length,e)}slice(e,n=this.length){[e,n]=ji(this,e,n);let r=[];return this.decompose(e,n,r,0),hn.from(r,n-e)}eq(e){if(e==this)return!0;if(e.length!=this.length||e.lines!=this.lines)return!1;let n=this.scanIdentical(e,1),r=this.length-this.scanIdentical(e,-1),i=new Ns(this),s=new Ns(e);for(let o=n,l=n;;){if(i.next(o),s.next(o),o=0,i.lineBreak!=s.lineBreak||i.done!=s.done||i.value!=s.value)return!1;if(l+=i.value.length,i.done||l>=r)return!0}}iter(e=1){return new Ns(this,e)}iterRange(e,n=this.length){return new fy(this,e,n)}iterLines(e,n){let r;if(e==null)r=this.iter();else{n==null&&(n=this.lines+1);let i=this.line(e).from;r=this.iterRange(i,Math.max(i,n==this.lines+1?this.length:n<=1?0:this.line(n-1).to))}return new dy(r)}toString(){return this.sliceString(0)}toJSON(){let e=[];return this.flatten(e),e}constructor(){}static of(e){if(e.length==0)throw new RangeError("A document must have at least one line");return e.length==1&&!e[0]?te.empty:e.length<=32?new Ie(e):hn.from(Ie.split(e,[]))}}class Ie extends te{constructor(e,n=yb(e)){super(),this.text=e,this.length=n}get lines(){return this.text.length}get children(){return null}lineInner(e,n,r,i){for(let s=0;;s++){let o=this.text[s],l=i+o.length;if((n?r:l)>=e)return new wb(i,l,r,o);i=l+1,r++}}decompose(e,n,r,i){let s=e<=0&&n>=this.length?this:new Ie(hm(this.text,e,n),Math.min(n,this.length)-Math.max(0,e));if(i&1){let o=r.pop(),l=Dl(s.text,o.text.slice(),0,s.length);if(l.length<=32)r.push(new Ie(l,o.length+s.length));else{let a=l.length>>1;r.push(new Ie(l.slice(0,a)),new Ie(l.slice(a)))}}else r.push(s)}replace(e,n,r){if(!(r instanceof Ie))return super.replace(e,n,r);[e,n]=ji(this,e,n);let i=Dl(this.text,Dl(r.text,hm(this.text,0,e)),n),s=this.length+r.length-(n-e);return i.length<=32?new Ie(i,s):hn.from(Ie.split(i,[]),s)}sliceString(e,n=this.length,r=`
`){[e,n]=ji(this,e,n);let i="";for(let s=0,o=0;s<=n&&o<this.text.length;o++){let l=this.text[o],a=s+l.length;s>e&&o&&(i+=r),e<a&&n>s&&(i+=l.slice(Math.max(0,e-s),n-s)),s=a+1}return i}flatten(e){for(let n of this.text)e.push(n)}scanIdentical(){return 0}static split(e,n){let r=[],i=-1;for(let s of e)r.push(s),i+=s.length+1,r.length==32&&(n.push(new Ie(r,i)),r=[],i=-1);return i>-1&&n.push(new Ie(r,i)),n}}class hn extends te{constructor(e,n){super(),this.children=e,this.length=n,this.lines=0;for(let r of e)this.lines+=r.lines}lineInner(e,n,r,i){for(let s=0;;s++){let o=this.children[s],l=i+o.length,a=r+o.lines-1;if((n?a:l)>=e)return o.lineInner(e,n,r,i);i=l+1,r=a+1}}decompose(e,n,r,i){for(let s=0,o=0;o<=n&&s<this.children.length;s++){let l=this.children[s],a=o+l.length;if(e<=a&&n>=o){let c=i&((o<=e?1:0)|(a>=n?2:0));o>=e&&a<=n&&!c?r.push(l):l.decompose(e-o,n-o,r,c)}o=a+1}}replace(e,n,r){if([e,n]=ji(this,e,n),r.lines<this.lines)for(let i=0,s=0;i<this.children.length;i++){let o=this.children[i],l=s+o.length;if(e>=s&&n<=l){let a=o.replace(e-s,n-s,r),c=this.lines-o.lines+a.lines;if(a.lines<c>>4&&a.lines>c>>6){let u=this.children.slice();return u[i]=a,new hn(u,this.length-(n-e)+r.length)}return super.replace(s,l,a)}s=l+1}return super.replace(e,n,r)}sliceString(e,n=this.length,r=`
`){[e,n]=ji(this,e,n);let i="";for(let s=0,o=0;s<this.children.length&&o<=n;s++){let l=this.children[s],a=o+l.length;o>e&&s&&(i+=r),e<a&&n>o&&(i+=l.sliceString(e-o,n-o,r)),o=a+1}return i}flatten(e){for(let n of this.children)n.flatten(e)}scanIdentical(e,n){if(!(e instanceof hn))return 0;let r=0,[i,s,o,l]=n>0?[0,0,this.children.length,e.children.length]:[this.children.length-1,e.children.length-1,-1,-1];for(;;i+=n,s+=n){if(i==o||s==l)return r;let a=this.children[i],c=e.children[s];if(a!=c)return r+a.scanIdentical(c,n);r+=a.length+1}}static from(e,n=e.reduce((r,i)=>r+i.length+1,-1)){let r=0;for(let d of e)r+=d.lines;if(r<32){let d=[];for(let p of e)p.flatten(d);return new Ie(d,n)}let i=Math.max(32,r>>5),s=i<<1,o=i>>1,l=[],a=0,c=-1,u=[];function h(d){let p;if(d.lines>s&&d instanceof hn)for(let m of d.children)h(m);else d.lines>o&&(a>o||!a)?(f(),l.push(d)):d instanceof Ie&&a&&(p=u[u.length-1])instanceof Ie&&d.lines+p.lines<=32?(a+=d.lines,c+=d.length+1,u[u.length-1]=new Ie(p.text.concat(d.text),p.length+1+d.length)):(a+d.lines>i&&f(),a+=d.lines,c+=d.length+1,u.push(d))}function f(){a!=0&&(l.push(u.length==1?u[0]:hn.from(u,c)),c=-1,a=u.length=0)}for(let d of e)h(d);return f(),l.length==1?l[0]:new hn(l,n)}}te.empty=new Ie([""],0);function yb(t){let e=-1;for(let n of t)e+=n.length+1;return e}function Dl(t,e,n=0,r=1e9){for(let i=0,s=0,o=!0;s<t.length&&i<=r;s++){let l=t[s],a=i+l.length;a>=n&&(a>r&&(l=l.slice(0,r-i)),i<n&&(l=l.slice(n-i)),o?(e[e.length-1]+=l,o=!1):e.push(l)),i=a+1}return e}function hm(t,e,n){return Dl(t,[""],e,n)}class Ns{constructor(e,n=1){this.dir=n,this.done=!1,this.lineBreak=!1,this.value="",this.nodes=[e],this.offsets=[n>0?1:(e instanceof Ie?e.text.length:e.children.length)<<1]}nextInner(e,n){for(this.done=this.lineBreak=!1;;){let r=this.nodes.length-1,i=this.nodes[r],s=this.offsets[r],o=s>>1,l=i instanceof Ie?i.text.length:i.children.length;if(o==(n>0?l:0)){if(r==0)return this.done=!0,this.value="",this;n>0&&this.offsets[r-1]++,this.nodes.pop(),this.offsets.pop()}else if((s&1)==(n>0?0:1)){if(this.offsets[r]+=n,e==0)return this.lineBreak=!0,this.value=`
`,this;e--}else if(i instanceof Ie){let a=i.text[o+(n<0?-1:0)];if(this.offsets[r]+=n,a.length>Math.max(0,e))return this.value=e==0?a:n>0?a.slice(e):a.slice(0,a.length-e),this;e-=a.length}else{let a=i.children[o+(n<0?-1:0)];e>a.length?(e-=a.length,this.offsets[r]+=n):(n<0&&this.offsets[r]--,this.nodes.push(a),this.offsets.push(n>0?1:(a instanceof Ie?a.text.length:a.children.length)<<1))}}}next(e=0){return e<0&&(this.nextInner(-e,-this.dir),e=this.value.length),this.nextInner(e,this.dir)}}class fy{constructor(e,n,r){this.value="",this.done=!1,this.cursor=new Ns(e,n>r?-1:1),this.pos=n>r?e.length:0,this.from=Math.min(n,r),this.to=Math.max(n,r)}nextInner(e,n){if(n<0?this.pos<=this.from:this.pos>=this.to)return this.value="",this.done=!0,this;e+=Math.max(0,n<0?this.pos-this.to:this.from-this.pos);let r=n<0?this.pos-this.from:this.to-this.pos;e>r&&(e=r),r-=e;let{value:i}=this.cursor.next(e);return this.pos+=(i.length+e)*n,this.value=i.length<=r?i:n<0?i.slice(i.length-r):i.slice(0,r),this.done=!this.value,this}next(e=0){return e<0?e=Math.max(e,this.from-this.pos):e>0&&(e=Math.min(e,this.to-this.pos)),this.nextInner(e,this.cursor.dir)}get lineBreak(){return this.cursor.lineBreak&&this.value!=""}}class dy{constructor(e){this.inner=e,this.afterBreak=!0,this.value="",this.done=!1}next(e=0){let{done:n,lineBreak:r,value:i}=this.inner.next(e);return n&&this.afterBreak?(this.value="",this.afterBreak=!1):n?(this.done=!0,this.value=""):r?this.afterBreak?this.value="":(this.afterBreak=!0,this.next()):(this.value=i,this.afterBreak=!1),this}get lineBreak(){return!1}}typeof Symbol<"u"&&(te.prototype[Symbol.iterator]=function(){return this.iter()},Ns.prototype[Symbol.iterator]=fy.prototype[Symbol.iterator]=dy.prototype[Symbol.iterator]=function(){return this});class wb{constructor(e,n,r,i){this.from=e,this.to=n,this.number=r,this.text=i}get length(){return this.to-this.from}}function ji(t,e,n){return e=Math.max(0,Math.min(t.length,e)),[e,Math.max(e,Math.min(t.length,n))]}function qe(t,e,n=!0,r=!0){return gb(t,e,n,r)}function Sb(t){return t>=56320&&t<57344}function kb(t){return t>=55296&&t<56320}function ft(t,e){let n=t.charCodeAt(e);if(!kb(n)||e+1==t.length)return n;let r=t.charCodeAt(e+1);return Sb(r)?(n-55296<<10)+(r-56320)+65536:n}function sd(t){return t<=65535?String.fromCharCode(t):(t-=65536,String.fromCharCode((t>>10)+55296,(t&1023)+56320))}function fn(t){return t<65536?1:2}const mh=/\r\n?|\n/;var it=function(t){return t[t.Simple=0]="Simple",t[t.TrackDel=1]="TrackDel",t[t.TrackBefore=2]="TrackBefore",t[t.TrackAfter=3]="TrackAfter",t}(it||(it={}));class kn{constructor(e){this.sections=e}get length(){let e=0;for(let n=0;n<this.sections.length;n+=2)e+=this.sections[n];return e}get newLength(){let e=0;for(let n=0;n<this.sections.length;n+=2){let r=this.sections[n+1];e+=r<0?this.sections[n]:r}return e}get empty(){return this.sections.length==0||this.sections.length==2&&this.sections[1]<0}iterGaps(e){for(let n=0,r=0,i=0;n<this.sections.length;){let s=this.sections[n++],o=this.sections[n++];o<0?(e(r,i,s),i+=s):i+=o,r+=s}}iterChangedRanges(e,n=!1){gh(this,e,n)}get invertedDesc(){let e=[];for(let n=0;n<this.sections.length;){let r=this.sections[n++],i=this.sections[n++];i<0?e.push(r,i):e.push(i,r)}return new kn(e)}composeDesc(e){return this.empty?e:e.empty?this:py(this,e)}mapDesc(e,n=!1){return e.empty?this:vh(this,e,n)}mapPos(e,n=-1,r=it.Simple){let i=0,s=0;for(let o=0;o<this.sections.length;){let l=this.sections[o++],a=this.sections[o++],c=i+l;if(a<0){if(c>e)return s+(e-i);s+=l}else{if(r!=it.Simple&&c>=e&&(r==it.TrackDel&&i<e&&c>e||r==it.TrackBefore&&i<e||r==it.TrackAfter&&c>e))return null;if(c>e||c==e&&n<0&&!l)return e==i||n<0?s:s+a;s+=a}i=c}if(e>i)throw new RangeError(`Position ${e} is out of range for changeset of length ${i}`);return s}touchesRange(e,n=e){for(let r=0,i=0;r<this.sections.length&&i<=n;){let s=this.sections[r++],o=this.sections[r++],l=i+s;if(o>=0&&i<=n&&l>=e)return i<e&&l>n?"cover":!0;i=l}return!1}toString(){let e="";for(let n=0;n<this.sections.length;){let r=this.sections[n++],i=this.sections[n++];e+=(e?" ":"")+r+(i>=0?":"+i:"")}return e}toJSON(){return this.sections}static fromJSON(e){if(!Array.isArray(e)||e.length%2||e.some(n=>typeof n!="number"))throw new RangeError("Invalid JSON representation of ChangeDesc");return new kn(e)}static create(e){return new kn(e)}}class De extends kn{constructor(e,n){super(e),this.inserted=n}apply(e){if(this.length!=e.length)throw new RangeError("Applying change set to a document with the wrong length");return gh(this,(n,r,i,s,o)=>e=e.replace(i,i+(r-n),o),!1),e}mapDesc(e,n=!1){return vh(this,e,n,!0)}invert(e){let n=this.sections.slice(),r=[];for(let i=0,s=0;i<n.length;i+=2){let o=n[i],l=n[i+1];if(l>=0){n[i]=l,n[i+1]=o;let a=i>>1;for(;r.length<a;)r.push(te.empty);r.push(o?e.slice(s,s+o):te.empty)}s+=o}return new De(n,r)}compose(e){return this.empty?e:e.empty?this:py(this,e,!0)}map(e,n=!1){return e.empty?this:vh(this,e,n,!0)}iterChanges(e,n=!1){gh(this,e,n)}get desc(){return kn.create(this.sections)}filter(e){let n=[],r=[],i=[],s=new oo(this);e:for(let o=0,l=0;;){let a=o==e.length?1e9:e[o++];for(;l<a||l==a&&s.len==0;){if(s.done)break e;let u=Math.min(s.len,a-l);Xe(i,u,-1);let h=s.ins==-1?-1:s.off==0?s.ins:0;Xe(n,u,h),h>0&&Xn(r,n,s.text),s.forward(u),l+=u}let c=e[o++];for(;l<c;){if(s.done)break e;let u=Math.min(s.len,c-l);Xe(n,u,-1),Xe(i,u,s.ins==-1?-1:s.off==0?s.ins:0),s.forward(u),l+=u}}return{changes:new De(n,r),filtered:kn.create(i)}}toJSON(){let e=[];for(let n=0;n<this.sections.length;n+=2){let r=this.sections[n],i=this.sections[n+1];i<0?e.push(r):i==0?e.push([r]):e.push([r].concat(this.inserted[n>>1].toJSON()))}return e}static of(e,n,r){let i=[],s=[],o=0,l=null;function a(u=!1){if(!u&&!i.length)return;o<n&&Xe(i,n-o,-1);let h=new De(i,s);l=l?l.compose(h.map(l)):h,i=[],s=[],o=0}function c(u){if(Array.isArray(u))for(let h of u)c(h);else if(u instanceof De){if(u.length!=n)throw new RangeError(`Mismatched change set length (got ${u.length}, expected ${n})`);a(),l=l?l.compose(u.map(l)):u}else{let{from:h,to:f=h,insert:d}=u;if(h>f||h<0||f>n)throw new RangeError(`Invalid change range ${h} to ${f} (in doc of length ${n})`);let p=d?typeof d=="string"?te.of(d.split(r||mh)):d:te.empty,m=p.length;if(h==f&&m==0)return;h<o&&a(),h>o&&Xe(i,h-o,-1),Xe(i,f-h,m),Xn(s,i,p),o=f}}return c(e),a(!l),l}static empty(e){return new De(e?[e,-1]:[],[])}static fromJSON(e){if(!Array.isArray(e))throw new RangeError("Invalid JSON representation of ChangeSet");let n=[],r=[];for(let i=0;i<e.length;i++){let s=e[i];if(typeof s=="number")n.push(s,-1);else{if(!Array.isArray(s)||typeof s[0]!="number"||s.some((o,l)=>l&&typeof o!="string"))throw new RangeError("Invalid JSON representation of ChangeSet");if(s.length==1)n.push(s[0],0);else{for(;r.length<i;)r.push(te.empty);r[i]=te.of(s.slice(1)),n.push(s[0],r[i].length)}}}return new De(n,r)}static createSet(e,n){return new De(e,n)}}function Xe(t,e,n,r=!1){if(e==0&&n<=0)return;let i=t.length-2;i>=0&&n<=0&&n==t[i+1]?t[i]+=e:i>=0&&e==0&&t[i]==0?t[i+1]+=n:r?(t[i]+=e,t[i+1]+=n):t.push(e,n)}function Xn(t,e,n){if(n.length==0)return;let r=e.length-2>>1;if(r<t.length)t[t.length-1]=t[t.length-1].append(n);else{for(;t.length<r;)t.push(te.empty);t.push(n)}}function gh(t,e,n){let r=t.inserted;for(let i=0,s=0,o=0;o<t.sections.length;){let l=t.sections[o++],a=t.sections[o++];if(a<0)i+=l,s+=l;else{let c=i,u=s,h=te.empty;for(;c+=l,u+=a,a&&r&&(h=h.append(r[o-2>>1])),!(n||o==t.sections.length||t.sections[o+1]<0);)l=t.sections[o++],a=t.sections[o++];e(i,c,s,u,h),i=c,s=u}}}function vh(t,e,n,r=!1){let i=[],s=r?[]:null,o=new oo(t),l=new oo(e);for(let a=-1;;){if(o.done&&l.len||l.done&&o.len)throw new Error("Mismatched change set lengths");if(o.ins==-1&&l.ins==-1){let c=Math.min(o.len,l.len);Xe(i,c,-1),o.forward(c),l.forward(c)}else if(l.ins>=0&&(o.ins<0||a==o.i||o.off==0&&(l.len<o.len||l.len==o.len&&!n))){let c=l.len;for(Xe(i,l.ins,-1);c;){let u=Math.min(o.len,c);o.ins>=0&&a<o.i&&o.len<=u&&(Xe(i,0,o.ins),s&&Xn(s,i,o.text),a=o.i),o.forward(u),c-=u}l.next()}else if(o.ins>=0){let c=0,u=o.len;for(;u;)if(l.ins==-1){let h=Math.min(u,l.len);c+=h,u-=h,l.forward(h)}else if(l.ins==0&&l.len<u)u-=l.len,l.next();else break;Xe(i,c,a<o.i?o.ins:0),s&&a<o.i&&Xn(s,i,o.text),a=o.i,o.forward(o.len-u)}else{if(o.done&&l.done)return s?De.createSet(i,s):kn.create(i);throw new Error("Mismatched change set lengths")}}}function py(t,e,n=!1){let r=[],i=n?[]:null,s=new oo(t),o=new oo(e);for(let l=!1;;){if(s.done&&o.done)return i?De.createSet(r,i):kn.create(r);if(s.ins==0)Xe(r,s.len,0,l),s.next();else if(o.len==0&&!o.done)Xe(r,0,o.ins,l),i&&Xn(i,r,o.text),o.next();else{if(s.done||o.done)throw new Error("Mismatched change set lengths");{let a=Math.min(s.len2,o.len),c=r.length;if(s.ins==-1){let u=o.ins==-1?-1:o.off?0:o.ins;Xe(r,a,u,l),i&&u&&Xn(i,r,o.text)}else o.ins==-1?(Xe(r,s.off?0:s.len,a,l),i&&Xn(i,r,s.textBit(a))):(Xe(r,s.off?0:s.len,o.off?0:o.ins,l),i&&!o.off&&Xn(i,r,o.text));l=(s.ins>a||o.ins>=0&&o.len>a)&&(l||r.length>c),s.forward2(a),o.forward(a)}}}}class oo{constructor(e){this.set=e,this.i=0,this.next()}next(){let{sections:e}=this.set;this.i<e.length?(this.len=e[this.i++],this.ins=e[this.i++]):(this.len=0,this.ins=-2),this.off=0}get done(){return this.ins==-2}get len2(){return this.ins<0?this.len:this.ins}get text(){let{inserted:e}=this.set,n=this.i-2>>1;return n>=e.length?te.empty:e[n]}textBit(e){let{inserted:n}=this.set,r=this.i-2>>1;return r>=n.length&&!e?te.empty:n[r].slice(this.off,e==null?void 0:this.off+e)}forward(e){e==this.len?this.next():(this.len-=e,this.off+=e)}forward2(e){this.ins==-1?this.forward(e):e==this.ins?this.next():(this.ins-=e,this.off+=e)}}class $n{constructor(e,n,r,i){this.from=e,this.to=n,this.flags=r,this.goalColumn=i}get anchor(){return this.flags&32?this.to:this.from}get head(){return this.flags&32?this.from:this.to}get empty(){return this.from==this.to}get assoc(){return this.flags&8?-1:this.flags&16?1:0}get undirectional(){return(this.flags&64)>0}get bidiLevel(){let e=this.flags&7;return e==7?null:e}map(e,n=-1){let r,i;return this.empty?r=i=e.mapPos(this.from,n):(r=e.mapPos(this.from,1),i=e.mapPos(this.to,-1)),r==this.from&&i==this.to?this:new $n(r,i,this.flags,this.goalColumn)}extend(e,n=e,r=0){if(e<=this.anchor&&n>=this.anchor)return R.range(e,n,void 0,void 0,r);let i=Math.abs(e-this.anchor)>Math.abs(n-this.anchor)?e:n;return R.range(this.anchor,i,void 0,void 0,r)}eq(e,n=!1){return this.anchor==e.anchor&&this.head==e.head&&this.goalColumn==e.goalColumn&&(!n||!this.empty||this.assoc==e.assoc)}toJSON(){return{anchor:this.anchor,head:this.head}}static fromJSON(e){if(!e||typeof e.anchor!="number"||typeof e.head!="number")throw new RangeError("Invalid JSON representation for SelectionRange");return R.range(e.anchor,e.head)}static create(e,n,r,i){return new $n(e,n,r,i)}}class R{constructor(e,n){this.ranges=e,this.mainIndex=n}map(e,n=-1){return e.empty?this:R.create(this.ranges.map(r=>r.map(e,n)),this.mainIndex)}eq(e,n=!1){if(this.ranges.length!=e.ranges.length||this.mainIndex!=e.mainIndex)return!1;for(let r=0;r<this.ranges.length;r++)if(!this.ranges[r].eq(e.ranges[r],n))return!1;return!0}get main(){return this.ranges[this.mainIndex]}asSingle(){return this.ranges.length==1?this:new R([this.main],0)}addRange(e,n=!0){return R.create([e].concat(this.ranges),n?0:this.mainIndex+1)}replaceRange(e,n=this.mainIndex){let r=this.ranges.slice();return r[n]=e,R.create(r,this.mainIndex)}toJSON(){return{ranges:this.ranges.map(e=>e.toJSON()),main:this.mainIndex}}static fromJSON(e){if(!e||!Array.isArray(e.ranges)||typeof e.main!="number"||e.main>=e.ranges.length)throw new RangeError("Invalid JSON representation for EditorSelection");return new R(e.ranges.map(n=>$n.fromJSON(n)),e.main)}static single(e,n=e){return new R([R.range(e,n)],0)}static create(e,n=0){if(e.length==0)throw new RangeError("A selection needs at least one range");for(let r=0,i=0;i<e.length;i++){let s=e[i];if(s.empty?s.from<=r:s.from<r)return R.normalized(e.slice(),n);r=s.to}return new R(e,n)}static cursor(e,n=0,r,i){return $n.create(e,e,(n==0?0:n<0?8:16)|(r==null?7:Math.min(6,r)),i)}static range(e,n,r,i,s){let o=i==null?7:Math.min(6,i);return!s&&e!=n&&(s=n<e?1:-1),s&&(o|=s<0?8:16),n<e?$n.create(n,e,o|32,r):$n.create(e,n,o,r)}static undirectionalRange(e,n){return $n.create(e,n,64,void 0)}static normalized(e,n=0){let r=e[n];e.sort((i,s)=>i.from-s.from),n=e.indexOf(r);for(let i=1;i<e.length;i++){let s=e[i],o=e[i-1];if(s.empty?s.from<=o.to:s.from<o.to){let l=o.from,a=Math.max(s.to,o.to);i<=n&&n--,e.splice(--i,2,s.anchor>s.head?R.range(a,l):R.range(l,a))}}return new R(e,n)}}function my(t,e){for(let n of t.ranges)if(n.to>e)throw new RangeError("Selection points outside of document")}let od=0;class z{constructor(e,n,r,i,s){this.combine=e,this.compareInput=n,this.compare=r,this.isStatic=i,this.id=od++,this.default=e([]),this.extensions=typeof s=="function"?s(this):s}get reader(){return this}static define(e={}){return new z(e.combine||(n=>n),e.compareInput||((n,r)=>n===r),e.compare||(e.combine?(n,r)=>n===r:ld),!!e.static,e.enables)}of(e){return new Ol([],this,0,e)}compute(e,n){if(this.isStatic)throw new Error("Can't compute a static facet");return new Ol(e,this,1,n)}computeN(e,n){if(this.isStatic)throw new Error("Can't compute a static facet");return new Ol(e,this,2,n)}from(e,n){return n||(n=r=>r),this.compute([e],r=>n(r.field(e)))}}function ld(t,e){return t==e||t.length==e.length&&t.every((n,r)=>n===e[r])}class Ol{constructor(e,n,r,i){this.dependencies=e,this.facet=n,this.type=r,this.value=i,this.id=od++}dynamicSlot(e){var n;let r=this.value,i=this.facet.compareInput,s=this.id,o=e[s]>>1,l=this.type==2,a=!1,c=!1,u=[];for(let h of this.dependencies)h=="doc"?a=!0:h=="selection"?c=!0:((n=e[h.id])!==null&&n!==void 0?n:1)&1||u.push(e[h.id]);return{create(h){return h.values[o]=r(h),1},update(h,f){if(a&&f.docChanged||c&&(f.docChanged||f.selection)||yh(h,u)){let d=r(h);if(l?!fm(d,h.values[o],i):!i(d,h.values[o]))return h.values[o]=d,1}return 0},reconfigure:(h,f)=>{let d,p=f.config.address[s];if(p!=null){let m=pa(f,p);if(this.dependencies.every(w=>w instanceof z?f.facet(w)===h.facet(w):w instanceof ze?f.field(w,!1)==h.field(w,!1):!0)||(l?fm(d=r(h),m,i):i(d=r(h),m)))return h.values[o]=m,0}else d=r(h);return h.values[o]=d,1}}}get extension(){return this}}function fm(t,e,n){if(t.length!=e.length)return!1;for(let r=0;r<t.length;r++)if(!n(t[r],e[r]))return!1;return!0}function yh(t,e){let n=!1;for(let r of e)Bs(t,r)&1&&(n=!0);return n}function xb(t,e,n){let r=n.map(a=>t[a.id]),i=n.map(a=>a.type),s=r.filter(a=>!(a&1)),o=t[e.id]>>1;function l(a){let c=[];for(let u=0;u<r.length;u++){let h=pa(a,r[u]);if(i[u]==2)for(let f of h)c.push(f);else c.push(h)}return e.combine(c)}return{create(a){for(let c of r)Bs(a,c);return a.values[o]=l(a),1},update(a,c){if(!yh(a,s))return 0;let u=l(a);return e.compare(u,a.values[o])?0:(a.values[o]=u,1)},reconfigure(a,c){let u=yh(a,r),h=c.config.facets[e.id],f=c.facet(e);if(h&&!u&&ld(n,h))return a.values[o]=f,0;let d=l(a);return e.compare(d,f)?(a.values[o]=f,0):(a.values[o]=d,1)}}}const Jo=z.define({static:!0});class ze{constructor(e,n,r,i,s){this.id=e,this.createF=n,this.updateF=r,this.compareF=i,this.spec=s,this.provides=void 0}static define(e){let n=new ze(od++,e.create,e.update,e.compare||((r,i)=>r===i),e);return e.provide&&(n.provides=e.provide(n)),n}create(e){let n=e.facet(Jo).find(r=>r.field==this);return((n==null?void 0:n.create)||this.createF)(e)}slot(e){let n=e[this.id]>>1;return{create:r=>(r.values[n]=this.create(r),1),update:(r,i)=>{let s=r.values[n],o=this.updateF(s,i);return this.compareF(s,o)?0:(r.values[n]=o,1)},reconfigure:(r,i)=>{let s=r.facet(Jo),o=i.facet(Jo),l;return(l=s.find(a=>a.field==this))&&l!=o.find(a=>a.field==this)?(r.values[n]=l.create(r),1):i.config.address[this.id]!=null?(r.values[n]=i.field(this),0):(r.values[n]=this.create(r),1)}}}init(e){return[this,Jo.of({field:this,create:e})]}get extension(){return this}}const Ir={lowest:4,low:3,default:2,high:1,highest:0};function ds(t){return e=>new gy(e,t)}const Zr={highest:ds(Ir.highest),high:ds(Ir.high),default:ds(Ir.default),low:ds(Ir.low),lowest:ds(Ir.lowest)};class gy{constructor(e,n){this.inner=e,this.prec=n}get extension(){return this}}class Xa{of(e){return new wh(this,e)}reconfigure(e){return Xa.reconfigure.of({compartment:this,extension:e})}get(e){return e.config.compartments.get(this)}}class wh{constructor(e,n){this.compartment=e,this.inner=n}get extension(){return this}}class da{constructor(e,n,r,i,s,o){for(this.base=e,this.compartments=n,this.dynamicSlots=r,this.address=i,this.staticValues=s,this.facets=o,this.statusTemplate=[];this.statusTemplate.length<r.length;)this.statusTemplate.push(0)}staticFacet(e){let n=this.address[e.id];return n==null?e.default:this.staticValues[n>>1]}static resolve(e,n,r){let i=[],s=Object.create(null),o=new Map;for(let f of Cb(e,n,o))f instanceof ze?i.push(f):(s[f.facet.id]||(s[f.facet.id]=[])).push(f);let l=Object.create(null),a=[],c=[];for(let f of i)l[f.id]=c.length<<1,c.push(d=>f.slot(d));let u=r==null?void 0:r.config.facets;for(let f in s){let d=s[f],p=d[0].facet,m=u&&u[f]||[];if(d.every(w=>w.type==0))if(l[p.id]=a.length<<1|1,ld(m,d))a.push(r.facet(p));else{let w=p.combine(d.map(g=>g.value));a.push(r&&p.compare(w,r.facet(p))?r.facet(p):w)}else{for(let w of d)w.type==0?(l[w.id]=a.length<<1|1,a.push(w.value)):(l[w.id]=c.length<<1,c.push(g=>w.dynamicSlot(g)));l[p.id]=c.length<<1,c.push(w=>xb(w,p,d))}}let h=c.map(f=>f(l));return new da(e,o,h,l,a,s)}}function Cb(t,e,n){let r=[[],[],[],[],[]],i=new Map;function s(o,l){let a=i.get(o);if(a!=null){if(a<=l)return;let c=r[a].indexOf(o);c>-1&&r[a].splice(c,1),o instanceof wh&&n.delete(o.compartment)}if(i.set(o,l),Array.isArray(o))for(let c of o)s(c,l);else if(o instanceof wh){if(n.has(o.compartment))throw new RangeError("Duplicate use of compartment in extensions");let c=e.get(o.compartment)||o.inner;n.set(o.compartment,c),s(c,l)}else if(o instanceof gy)s(o.inner,o.prec);else if(o instanceof ze)r[l].push(o),o.provides&&s(o.provides,l);else if(o instanceof Ol)r[l].push(o),o.facet.extensions&&s(o.facet.extensions,Ir.default);else{let c=o.extension;if(!c)throw new Error(`Unrecognized extension value in extension set (${o}).`);if(c==o)throw new Error(`Unrecognized extension value in extension set (${o}). This sometimes happens because multiple instances of @codemirror/state are loaded, breaking instanceof checks.`);s(c,l)}}return s(t,Ir.default),r.reduce((o,l)=>o.concat(l))}function Bs(t,e){if(e&1)return 2;let n=e>>1,r=t.status[n];if(r==4)throw new Error("Cyclic dependency between fields and/or facets");if(r&2)return r;t.status[n]=4;let i=t.computeSlot(t,t.config.dynamicSlots[n]);return t.status[n]=2|i}function pa(t,e){return e&1?t.config.staticValues[e>>1]:t.values[e>>1]}const vy=z.define(),Sh=z.define({combine:t=>t.some(e=>e),static:!0}),yy=z.define({combine:t=>t.length?t[0]:void 0,static:!0}),wy=z.define(),Sy=z.define(),ky=z.define(),xy=z.define({combine:t=>t.length?t[0]:!1});class _n{constructor(e,n){this.type=e,this.value=n}static define(){return new bb}}class bb{of(e){return new _n(this,e)}}class Tb{constructor(e){this.map=e}of(e){return new K(this,e)}}class K{constructor(e,n){this.type=e,this.value=n}map(e){let n=this.type.map(this.value,e);return n===void 0?void 0:n==this.value?this:new K(this.type,n)}is(e){return this.type==e}static define(e={}){return new Tb(e.map||(n=>n))}static mapEffects(e,n){if(!e.length)return e;let r=[];for(let i of e){let s=i.map(n);s&&r.push(s)}return r}}K.reconfigure=K.define();K.appendConfig=K.define();class Ne{constructor(e,n,r,i,s,o){this.startState=e,this.changes=n,this.selection=r,this.effects=i,this.annotations=s,this.scrollIntoView=o,this._doc=null,this._state=null,r&&my(r,n.newLength),s.some(l=>l.type==Ne.time)||(this.annotations=s.concat(Ne.time.of(Date.now())))}static create(e,n,r,i,s,o){return new Ne(e,n,r,i,s,o)}get newDoc(){return this._doc||(this._doc=this.changes.apply(this.startState.doc))}get newSelection(){return this.selection||this.startState.selection.map(this.changes)}get state(){return this._state||this.startState.applyTransaction(this),this._state}annotation(e){for(let n of this.annotations)if(n.type==e)return n.value}get docChanged(){return!this.changes.empty}get reconfigured(){return this.startState.config!=this.state.config}isUserEvent(e){let n=this.annotation(Ne.userEvent);return!!(n&&(n==e||n.length>e.length&&n.slice(0,e.length)==e&&n[e.length]=="."))}}Ne.time=_n.define();Ne.userEvent=_n.define();Ne.addToHistory=_n.define();Ne.remote=_n.define();function Rb(t,e){let n=[];for(let r=0,i=0;;){let s,o;if(r<t.length&&(i==e.length||e[i]>=t[r]))s=t[r++],o=t[r++];else if(i<e.length)s=e[i++],o=e[i++];else return n;!n.length||n[n.length-1]<s?n.push(s,o):n[n.length-1]<o&&(n[n.length-1]=o)}}function Cy(t,e,n){var r;let i,s,o;return n?(i=e.changes,s=De.empty(e.changes.length),o=t.changes.compose(e.changes)):(i=e.changes.map(t.changes),s=t.changes.mapDesc(e.changes,!0),o=t.changes.compose(i)),{changes:o,selection:e.selection?e.selection.map(s):(r=t.selection)===null||r===void 0?void 0:r.map(i),effects:K.mapEffects(t.effects,i).concat(K.mapEffects(e.effects,s)),annotations:t.annotations.length?t.annotations.concat(e.annotations):e.annotations,scrollIntoView:t.scrollIntoView||e.scrollIntoView}}function kh(t,e,n){let r=e.selection,i=Ii(e.annotations);return e.userEvent&&(i=i.concat(Ne.userEvent.of(e.userEvent))),{changes:e.changes instanceof De?e.changes:De.of(e.changes||[],n,t.facet(yy)),selection:r&&(r instanceof R?r:R.single(r.anchor,r.head)),effects:Ii(e.effects),annotations:i,scrollIntoView:!!e.scrollIntoView}}function by(t,e,n){let r=kh(t,e.length?e[0]:{},t.doc.length);e.length&&e[0].filter===!1&&(n=!1);for(let s=1;s<e.length;s++){e[s].filter===!1&&(n=!1);let o=!!e[s].sequential;r=Cy(r,kh(t,e[s],o?r.changes.newLength:t.doc.length),o)}let i=Ne.create(t,r.changes,r.selection,r.effects,r.annotations,r.scrollIntoView);return Ib(n?Mb(i):i)}function Mb(t){let e=t.startState,n=!0;for(let i of e.facet(wy)){let s=i(t);if(s===!1){n=!1;break}Array.isArray(s)&&(n=n===!0?s:Rb(n,s))}if(n!==!0){let i,s;if(n===!1)s=t.changes.invertedDesc,i=De.empty(e.doc.length);else{let o=t.changes.filter(n);i=o.changes,s=o.filtered.mapDesc(o.changes).invertedDesc}t=Ne.create(e,i,t.selection&&t.selection.map(s),K.mapEffects(t.effects,s),t.annotations,t.scrollIntoView)}let r=e.facet(Sy);for(let i=r.length-1;i>=0;i--){let s=r[i](t);s instanceof Ne?t=s:Array.isArray(s)&&s.length==1&&s[0]instanceof Ne?t=s[0]:t=by(e,Ii(s),!1)}return t}function Ib(t){let e=t.startState,n=e.facet(ky),r=t;for(let i=n.length-1;i>=0;i--){let s=n[i](t);s&&Object.keys(s).length&&(r=Cy(r,kh(e,s,t.changes.newLength),!0))}return r==t?t:Ne.create(e,t.changes,t.selection,r.effects,r.annotations,r.scrollIntoView)}const Ab=[];function Ii(t){return t==null?Ab:Array.isArray(t)?t:[t]}var we=function(t){return t[t.Word=0]="Word",t[t.Space=1]="Space",t[t.Other=2]="Other",t}(we||(we={}));const Pb=/[\u00df\u0587\u0590-\u05f4\u0600-\u06ff\u3040-\u309f\u30a0-\u30ff\u3400-\u4db5\u4e00-\u9fcc\uac00-\ud7af]/;let xh;try{xh=new RegExp("[\\p{Alphabetic}\\p{Number}_]","u")}catch{}function Eb(t){if(xh)return xh.test(t);for(let e=0;e<t.length;e++){let n=t[e];if(/\w/.test(n)||n>""&&(n.toUpperCase()!=n.toLowerCase()||Pb.test(n)))return!0}return!1}function Lb(t){return e=>{if(!/\S/.test(e))return we.Space;if(Eb(e))return we.Word;for(let n=0;n<t.length;n++)if(e.indexOf(t[n])>-1)return we.Word;return we.Other}}class ee{constructor(e,n,r,i,s,o){this.config=e,this.doc=n,this.selection=r,this.values=i,this.status=e.statusTemplate.slice(),this.computeSlot=s,o&&(o._state=this);for(let l=0;l<this.config.dynamicSlots.length;l++)Bs(this,l<<1);this.computeSlot=null}field(e,n=!0){let r=this.config.address[e.id];if(r==null){if(n)throw new RangeError("Field is not present in this state");return}return Bs(this,r),pa(this,r)}update(...e){return by(this,e,!0)}applyTransaction(e){let n=this.config,{base:r,compartments:i}=n;for(let l of e.effects)l.is(Xa.reconfigure)?(n&&(i=new Map,n.compartments.forEach((a,c)=>i.set(c,a)),n=null),i.set(l.value.compartment,l.value.extension)):l.is(K.reconfigure)?(n=null,r=l.value):l.is(K.appendConfig)&&(n=null,r=Ii(r).concat(l.value));let s;n?s=e.startState.values.slice():(n=da.resolve(r,i,this),s=new ee(n,this.doc,this.selection,n.dynamicSlots.map(()=>null),(a,c)=>c.reconfigure(a,this),null).values);let o=e.startState.facet(Sh)?e.newSelection:e.newSelection.asSingle();new ee(n,e.newDoc,o,s,(l,a)=>a.update(l,e),e)}replaceSelection(e){return typeof e=="string"&&(e=this.toText(e)),this.changeByRange(n=>({changes:{from:n.from,to:n.to,insert:e},range:R.cursor(n.from+e.length,-1)}))}changeByRange(e){let n=this.selection,r=e(n.ranges[0]),i=this.changes(r.changes),s=[r.range],o=Ii(r.effects);for(let l=1;l<n.ranges.length;l++){let a=e(n.ranges[l]),c=this.changes(a.changes),u=c.map(i);for(let f=0;f<l;f++)s[f]=s[f].map(u);let h=i.mapDesc(c,!0);s.push(a.range.map(h)),i=i.compose(u),o=K.mapEffects(o,u).concat(K.mapEffects(Ii(a.effects),h))}return{changes:i,selection:R.create(s,n.mainIndex),effects:o}}changes(e=[]){return e instanceof De?e:De.of(e,this.doc.length,this.facet(ee.lineSeparator))}toText(e){return te.of(e.split(this.facet(ee.lineSeparator)||mh))}sliceDoc(e=0,n=this.doc.length){return this.doc.sliceString(e,n,this.lineBreak)}facet(e){let n=this.config.address[e.id];return n==null?e.default:(Bs(this,n),pa(this,n))}toJSON(e){let n={doc:this.sliceDoc(),selection:this.selection.toJSON()};if(e)for(let r in e){let i=e[r];i instanceof ze&&this.config.address[i.id]!=null&&(n[r]=i.spec.toJSON(this.field(e[r]),this))}return n}static fromJSON(e,n={},r){if(!e||typeof e.doc!="string")throw new RangeError("Invalid JSON representation for EditorState");let i=[];if(r){for(let s in r)if(Object.prototype.hasOwnProperty.call(e,s)){let o=r[s],l=e[s];i.push(o.init(a=>o.spec.fromJSON(l,a)))}}return ee.create({doc:e.doc,selection:R.fromJSON(e.selection),extensions:n.extensions?i.concat([n.extensions]):i})}static create(e={}){let n=da.resolve(e.extensions||[],new Map),r=e.doc instanceof te?e.doc:te.of((e.doc||"").split(n.staticFacet(ee.lineSeparator)||mh)),i=e.selection?e.selection instanceof R?e.selection:R.single(e.selection.anchor,e.selection.head):R.single(0);return my(i,r.length),n.staticFacet(Sh)||(i=i.asSingle()),new ee(n,r,i,n.dynamicSlots.map(()=>null),(s,o)=>o.create(s),null)}get tabSize(){return this.facet(ee.tabSize)}get lineBreak(){return this.facet(ee.lineSeparator)||`
`}get readOnly(){return this.facet(xy)}phrase(e,...n){for(let r of this.facet(ee.phrases))if(Object.prototype.hasOwnProperty.call(r,e)){e=r[e];break}return n.length&&(e=e.replace(/\$(\$|\d*)/g,(r,i)=>{if(i=="$")return"$";let s=+(i||1);return!s||s>n.length?r:n[s-1]})),e}languageDataAt(e,n,r=-1){let i=[];for(let s of this.facet(vy))for(let o of s(this,n,r))Object.prototype.hasOwnProperty.call(o,e)&&i.push(o[e]);return i}charCategorizer(e){let n=this.languageDataAt("wordChars",e);return Lb(n.length?n[0]:"")}wordAt(e){let{text:n,from:r,length:i}=this.doc.lineAt(e),s=this.charCategorizer(e),o=e-r,l=e-r;for(;o>0;){let a=qe(n,o,!1);if(s(n.slice(a,o))!=we.Word)break;o=a}for(;l<i;){let a=qe(n,l);if(s(n.slice(l,a))!=we.Word)break;l=a}return o==l?null:R.range(o+r,l+r)}}ee.allowMultipleSelections=Sh;ee.tabSize=z.define({combine:t=>t.length?t[0]:4});ee.lineSeparator=yy;ee.readOnly=xy;ee.phrases=z.define({compare(t,e){let n=Object.keys(t),r=Object.keys(e);return n.length==r.length&&n.every(i=>t[i]==e[i])}});ee.languageData=vy;ee.changeFilter=wy;ee.transactionFilter=Sy;ee.transactionExtender=ky;Xa.reconfigure=K.define();function Zt(t,e,n={}){let r={};for(let i of t)for(let s of Object.keys(i)){let o=i[s],l=r[s];if(l===void 0)r[s]=o;else if(!(l===o||o===void 0))if(Object.hasOwnProperty.call(n,s))r[s]=n[s](l,o);else throw new Error("Config merge conflict for field "+s)}for(let i in e)r[i]===void 0&&(r[i]=e[i]);return r}class ur{eq(e){return this==e}range(e,n=e){return Ch.create(e,n,this)}}ur.prototype.startSide=ur.prototype.endSide=0;ur.prototype.point=!1;ur.prototype.mapMode=it.TrackDel;function ad(t,e){return t==e||t.constructor==e.constructor&&t.eq(e)}let Ch=class Ty{constructor(e,n,r){this.from=e,this.to=n,this.value=r}static create(e,n,r){return new Ty(e,n,r)}};function bh(t,e){return t.from-e.from||t.value.startSide-e.value.startSide}class cd{constructor(e,n,r,i){this.from=e,this.to=n,this.value=r,this.maxPoint=i}get length(){return ni(this.to)}findIndex(e,n,r,i=0){let s=r?this.to:this.from;for(let o=i,l=s.length;;){if(o==l)return o;let a=o+l>>1,c=s[a]-e||(r?this.value[a].endSide:this.value[a].startSide)-n;if(a==o)return c>=0?o:l;c>=0?l=a:o=a+1}}between(e,n,r,i){for(let s=this.findIndex(n,-1e9,!0),o=this.findIndex(r,1e9,!1,s);s<o;s++)if(i(this.from[s]+e,this.to[s]+e,this.value[s])===!1)return!1}map(e,n,r,i,s){let o=[],l=[],a=[],c=-1,u=-1;e:for(let h=0;h<this.value.length;h++){let f=this.value[h],d=this.from[h]+e,p=this.to[h]+e,m,w;if(d==p){let g=n.mapPos(d,f.startSide,f.mapMode);if(g==null||(m=w=g,f.startSide!=f.endSide&&(w=n.mapPos(d,f.endSide),w<m)))continue}else if(m=n.mapPos(d,f.startSide),w=n.mapPos(p,f.endSide),m>w||m==w&&f.startSide>0&&f.endSide<=0)continue;if(!((w-m||f.endSide-f.startSide)<0))if(c<0&&(c=m),f.point&&(u=Math.max(u,w-m)),(m-r||f.startSide-i)>=0)o.push(f),l.push(m-c),a.push(w-c),r=w,i=f.endSide;else{if(m==w)for(let g=o.length;g>0;g--){if((m-(a[g-1]+c)||f.startSide-o[g-1].endSide)>=0){o.splice(g,0,f),l.splice(g,0,m-c),a.splice(g,0,w-c);continue e}if((m-(l[g-1]+c)||f.endSide-o[g-1].startSide)>0)break}s(m,w,f)}}return{mapped:o.length?new cd(l,a,o,u):null,pos:c}}}class X{constructor(e,n,r,i){this.chunkPos=e,this.chunk=n,this.nextLayer=r,this.maxPoint=i}static create(e,n,r,i){return new X(e,n,r,i)}get length(){let e=this.chunk.length-1;return e<0?0:Math.max(this.chunkEnd(e),this.nextLayer.length)}get size(){if(this.isEmpty)return 0;let e=this.nextLayer.size;for(let n of this.chunk)e+=n.value.length;return e}chunkEnd(e){return this.chunkPos[e]+this.chunk[e].length}update(e){let{add:n=[],sort:r=!1,filterFrom:i=0,filterTo:s=this.length}=e,o=e.filter;if(n.length==0&&!o)return this;if(r&&(n=n.slice().sort(bh)),this.isEmpty)return n.length?X.of(n):this;let l=new Ry(this,null,-1).goto(0),a=0,c=[],u=new xn;for(;l.value||a<n.length;)if(a<n.length&&(l.from-n[a].from||l.startSide-n[a].value.startSide)>=0){let h=n[a++];u.addInner(h.from,h.to,h.value,!1)||c.push(h)}else l.rangeIndex==1&&l.chunkIndex<this.chunk.length&&(a==n.length||this.chunkEnd(l.chunkIndex)<n[a].from)&&(!o||i>this.chunkEnd(l.chunkIndex)||s<this.chunkPos[l.chunkIndex])&&u.addChunk(this.chunkPos[l.chunkIndex],this.chunk[l.chunkIndex])?l.nextChunk():((!o||i>l.to||s<l.from||o(l.from,l.to,l.value))&&(u.addInner(l.from,l.to,l.value,!1)||c.push(Ch.create(l.from,l.to,l.value))),l.next());return u.finishInner(this.nextLayer.isEmpty&&!c.length?X.empty:this.nextLayer.update({add:c,filter:o,filterFrom:i,filterTo:s}))}map(e){if(e.empty||this.isEmpty)return this;let n=[],r=[],i=-1,s,o=(a,c,u)=>{s||(s=new xn),s.addRange(a,c,u,!1)};for(let a=0;a<this.chunk.length;a++){let c=this.chunkPos[a],u=this.chunk[a],h=e.touchesRange(c,c+u.length);if(h===!1)i=Math.max(i,u.maxPoint),n.push(u),r.push(e.mapPos(c));else if(h===!0){let[f,d]=n.length?[ni(r)+ni(n).length,ni(ni(n).value).endSide]:[-1,-1],{mapped:p,pos:m}=u.map(c,e,f,d,o);p&&(i=Math.max(i,p.maxPoint),n.push(p),r.push(m))}}let l=this.nextLayer.map(e);return s&&(l=s.finishInner(l)),n.length==0?l:new X(r,n,l||X.empty,i)}between(e,n,r){if(!this.isEmpty){for(let i=0;i<this.chunk.length;i++){let s=this.chunkPos[i],o=this.chunk[i];if(n>=s&&e<=s+o.length&&o.between(s,e-s,n-s,r)===!1)return}this.nextLayer.between(e,n,r)}}iter(e=0){return lo.from([this]).goto(e)}get isEmpty(){return this.nextLayer==this}static iter(e,n=0){return lo.from(e).goto(n)}static compare(e,n,r,i,s=-1){let o=e.filter(h=>h.maxPoint>0||!h.isEmpty&&h.maxPoint>=s),l=n.filter(h=>h.maxPoint>0||!h.isEmpty&&h.maxPoint>=s),a=dm(o,l,r),c=new ps(o,a,s),u=new ps(l,a,s);r.iterGaps((h,f,d)=>pm(c,h,u,f,d,i)),r.empty&&r.length==0&&pm(c,0,u,0,0,i)}static eq(e,n,r=0,i){i==null&&(i=999999999);let s=e.filter(u=>!u.isEmpty&&n.indexOf(u)<0),o=n.filter(u=>!u.isEmpty&&e.indexOf(u)<0);if(s.length!=o.length)return!1;if(!s.length)return!0;let l=dm(s,o),a=new ps(s,l,0).goto(r),c=new ps(o,l,0).goto(r);for(;;){if(a.to!=c.to||!Th(a.active,c.active)||a.point&&(!c.point||!ad(a.point,c.point)))return!1;if(a.to>i)return!0;a.next(),c.next()}}static spans(e,n,r,i,s=-1){let o=new ps(e,null,s).goto(n),l=n,a=o.openStart;for(;;){let c=Math.min(o.to,r);if(o.point){let u=o.activeForPoint(o.to),h=o.pointFrom<n?u.length+1:o.point.startSide<0?u.length:Math.min(u.length,a);i.point(l,c,o.point,u,h,o.pointRank),a=Math.min(o.openEnd(c),u.length)}else c>l&&(i.span(l,c,o.active,a),a=o.openEnd(c));if(o.to>r)return a+(o.point&&o.to>r?1:0);l=o.to,o.next()}}static of(e,n=!1){let r=new xn;for(let i of e instanceof Ch?[e]:n?Db(e):e)r.add(i.from,i.to,i.value);return r.finish()}static join(e){if(!e.length)return X.empty;let n=ni(e);for(let r=e.length-2;r>=0;r--)for(let i=e[r];i!=X.empty;i=i.nextLayer)n=new X(i.chunkPos,i.chunk,n,Math.max(i.maxPoint,n.maxPoint));return n}}X.empty=new X([],[],null,-1);function ni(t){return t[t.length-1]}function Db(t){if(t.length>1)for(let e=t[0],n=1;n<t.length;n++){let r=t[n];if(bh(e,r)>0)return t.slice().sort(bh);e=r}return t}X.empty.nextLayer=X.empty;class xn{finishChunk(e){this.chunks.push(new cd(this.from,this.to,this.value,this.maxPoint)),this.chunkPos.push(this.chunkStart),this.chunkStart=-1,this.setMaxPoint=Math.max(this.setMaxPoint,this.maxPoint),this.maxPoint=-1,e&&(this.from=[],this.to=[],this.value=[])}constructor(){this.chunks=[],this.chunkPos=[],this.chunkStart=-1,this.last=null,this.lastFrom=-1e9,this.lastTo=-1e9,this.from=[],this.to=[],this.value=[],this.maxPoint=-1,this.setMaxPoint=-1,this.nextLayer=null}add(e,n,r){this.addRange(e,n,r,!0)}addRange(e,n,r,i){this.addInner(e,n,r,i)||(this.nextLayer||(this.nextLayer=new xn)).addRange(e,n,r,i)}addInner(e,n,r,i){let s=e-this.lastTo||r.startSide-this.last.endSide;if(i&&s<=0&&(e-this.lastFrom||r.startSide-this.last.startSide)<0)throw new Error("Ranges must be added sorted by `from` position and `startSide`");return s<0?!1:(this.from.length==250&&this.finishChunk(!0),this.chunkStart<0&&(this.chunkStart=e),this.from.push(e-this.chunkStart),this.to.push(n-this.chunkStart),this.last=r,this.lastFrom=e,this.lastTo=n,this.value.push(r),r.point&&(this.maxPoint=Math.max(this.maxPoint,n-e)),!0)}addChunk(e,n){if((e-this.lastTo||n.value[0].startSide-this.last.endSide)<0)return!1;this.from.length&&this.finishChunk(!0),this.setMaxPoint=Math.max(this.setMaxPoint,n.maxPoint),this.chunks.push(n),this.chunkPos.push(e);let r=n.value.length-1;return this.last=n.value[r],this.lastFrom=n.from[r]+e,this.lastTo=n.to[r]+e,!0}finish(){return this.finishInner(X.empty)}finishInner(e){if(this.from.length&&this.finishChunk(!1),this.chunks.length==0)return e;let n=X.create(this.chunkPos,this.chunks,this.nextLayer?this.nextLayer.finishInner(e):e,this.setMaxPoint);return this.from=null,n}}function dm(t,e,n){let r=new Map;for(let s of t)for(let o=0;o<s.chunk.length;o++)s.chunk[o].maxPoint<=0&&r.set(s.chunk[o],s.chunkPos[o]);let i=new Set;for(let s of e)for(let o=0;o<s.chunk.length;o++){let l=r.get(s.chunk[o]);l!=null&&(n?n.mapPos(l):l)==s.chunkPos[o]&&!(n!=null&&n.touchesRange(l,l+s.chunk[o].length))&&i.add(s.chunk[o])}return i}class Ry{constructor(e,n,r,i=0){this.layer=e,this.skip=n,this.minPoint=r,this.rank=i}get startSide(){return this.value?this.value.startSide:0}get endSide(){return this.value?this.value.endSide:0}goto(e,n=-1e9){return this.chunkIndex=this.rangeIndex=0,this.gotoInner(e,n,!1),this}gotoInner(e,n,r){for(;this.chunkIndex<this.layer.chunk.length;){let i=this.layer.chunk[this.chunkIndex];if(!(this.skip&&this.skip.has(i)||this.layer.chunkEnd(this.chunkIndex)<e||i.maxPoint<this.minPoint))break;this.chunkIndex++,r=!1}if(this.chunkIndex<this.layer.chunk.length){let i=this.layer.chunk[this.chunkIndex].findIndex(e-this.layer.chunkPos[this.chunkIndex],n,!0);(!r||this.rangeIndex<i)&&this.setRangeIndex(i)}this.next()}forward(e,n){(this.to-e||this.endSide-n)<0&&this.gotoInner(e,n,!0)}next(){for(;;)if(this.chunkIndex==this.layer.chunk.length){this.from=this.to=1e9,this.value=null;break}else{let e=this.layer.chunkPos[this.chunkIndex],n=this.layer.chunk[this.chunkIndex],r=e+n.from[this.rangeIndex];if(this.from=r,this.to=e+n.to[this.rangeIndex],this.value=n.value[this.rangeIndex],this.setRangeIndex(this.rangeIndex+1),this.minPoint<0||this.value.point&&this.to-this.from>=this.minPoint)break}}setRangeIndex(e){if(e==this.layer.chunk[this.chunkIndex].value.length){if(this.chunkIndex++,this.skip)for(;this.chunkIndex<this.layer.chunk.length&&this.skip.has(this.layer.chunk[this.chunkIndex]);)this.chunkIndex++;this.rangeIndex=0}else this.rangeIndex=e}nextChunk(){this.chunkIndex++,this.rangeIndex=0,this.next()}compare(e){return this.from-e.from||this.startSide-e.startSide||this.rank-e.rank||this.to-e.to||this.endSide-e.endSide}}class lo{constructor(e){this.heap=e}static from(e,n=null,r=-1){let i=[];for(let s=0;s<e.length;s++)for(let o=e[s];!o.isEmpty;o=o.nextLayer)o.maxPoint>=r&&i.push(new Ry(o,n,r,s));return i.length==1?i[0]:new lo(i)}get startSide(){return this.value?this.value.startSide:0}goto(e,n=-1e9){for(let r of this.heap)r.goto(e,n);for(let r=this.heap.length>>1;r>=0;r--)Uc(this.heap,r);return this.next(),this}forward(e,n){for(let r of this.heap)r.forward(e,n);for(let r=this.heap.length>>1;r>=0;r--)Uc(this.heap,r);(this.to-e||this.value.endSide-n)<0&&this.next()}next(){if(this.heap.length==0)this.from=this.to=1e9,this.value=null,this.rank=-1;else{let e=this.heap[0];this.from=e.from,this.to=e.to,this.value=e.value,this.rank=e.rank,e.value&&e.next(),Uc(this.heap,0)}}}function Uc(t,e){for(let n=t[e];;){let r=(e<<1)+1;if(r>=t.length)break;let i=t[r];if(r+1<t.length&&i.compare(t[r+1])>=0&&(i=t[r+1],r++),n.compare(i)<0)break;t[r]=n,t[e]=i,e=r}}class ps{constructor(e,n,r){this.minPoint=r,this.active=[],this.activeTo=[],this.activeRank=[],this.minActive=-1,this.point=null,this.pointFrom=0,this.pointRank=0,this.to=-1e9,this.endSide=0,this.openStart=-1,this.cursor=lo.from(e,n,r)}goto(e,n=-1e9){return this.cursor.goto(e,n),this.active.length=this.activeTo.length=this.activeRank.length=0,this.minActive=-1,this.to=e,this.endSide=n,this.openStart=-1,this.next(),this}forward(e,n){for(;this.minActive>-1&&(this.activeTo[this.minActive]-e||this.active[this.minActive].endSide-n)<0;)this.removeActive(this.minActive);this.cursor.forward(e,n)}removeActive(e){Zo(this.active,e),Zo(this.activeTo,e),Zo(this.activeRank,e),this.minActive=mm(this.active,this.activeTo)}addActive(e){let n=0,{value:r,to:i,rank:s}=this.cursor;for(;n<this.activeRank.length&&(s-this.activeRank[n]||i-this.activeTo[n])>0;)n++;el(this.active,n,r),el(this.activeTo,n,i),el(this.activeRank,n,s),e&&el(e,n,this.cursor.from),this.minActive=mm(this.active,this.activeTo)}next(){let e=this.to,n=this.point;this.point=null;let r=this.openStart<0?[]:null;for(;;){let i=this.minActive;if(i>-1&&(this.activeTo[i]-this.cursor.from||this.active[i].endSide-this.cursor.startSide)<0){if(this.activeTo[i]>e){this.to=this.activeTo[i],this.endSide=this.active[i].endSide;break}this.removeActive(i),r&&Zo(r,i)}else if(this.cursor.value)if(this.cursor.from>e){this.to=this.cursor.from,this.endSide=this.cursor.startSide;break}else{let s=this.cursor.value;if(!s.point)this.addActive(r),this.cursor.next();else if(n&&this.cursor.to==this.to&&this.cursor.from<this.cursor.to)this.cursor.next();else{this.point=s,this.pointFrom=this.cursor.from,this.pointRank=this.cursor.rank,this.to=this.cursor.to,this.endSide=s.endSide,this.cursor.next(),this.forward(this.to,this.endSide);break}}else{this.to=this.endSide=1e9;break}}if(r){this.openStart=0;for(let i=r.length-1;i>=0&&r[i]<e;i--)this.openStart++}}activeForPoint(e){if(!this.active.length)return this.active;let n=[];for(let r=this.active.length-1;r>=0&&!(this.activeRank[r]<this.pointRank);r--)(this.activeTo[r]>e||this.activeTo[r]==e&&this.active[r].endSide>=this.point.endSide)&&n.push(this.active[r]);return n.reverse()}openEnd(e){let n=0;for(let r=this.activeTo.length-1;r>=0&&this.activeTo[r]>e;r--)n++;return n}}function pm(t,e,n,r,i,s){t.goto(e),n.goto(r);let o=r+i,l=r,a=r-e,c=!!s.boundChange;for(let u=!1;;){let h=t.to+a-n.to,f=h||t.endSide-n.endSide,d=f<0?t.to+a:n.to,p=Math.min(d,o);if(t.point||n.point?(t.point&&n.point&&ad(t.point,n.point)&&Th(t.activeForPoint(t.to),n.activeForPoint(n.to))||s.comparePoint(l,p,t.point,n.point),u=!1):(u&&(s.boundChange(l),u=!1),p>l&&!Th(t.active,n.active)&&s.compareRange(l,p,t.active,n.active),c&&p<o&&(h||t.openEnd(d)!=n.openEnd(d))&&(u=!0)),d>o)break;l=d,f<=0&&t.next(),f>=0&&n.next()}}function Th(t,e){if(t.length!=e.length)return!1;for(let n=0;n<t.length;n++)if(t[n]!=e[n]&&!ad(t[n],e[n]))return!1;return!0}function Zo(t,e){for(let n=e,r=t.length-1;n<r;n++)t[n]=t[n+1];t.pop()}function el(t,e,n){for(let r=t.length-1;r>=e;r--)t[r+1]=t[r];t[e]=n}function mm(t,e){let n=-1,r=1e9;for(let i=0;i<e.length;i++)(e[i]-r||t[i].endSide-t[n].endSide)<0&&(n=i,r=e[i]);return n}function es(t,e,n=t.length){let r=0;for(let i=0;i<n&&i<t.length;)t.charCodeAt(i)==9?(r+=e-r%e,i++):(r++,i=qe(t,i));return r}function Rh(t,e,n,r){for(let i=0,s=0;;){if(s>=e)return i;if(i==t.length)break;s+=t.charCodeAt(i)==9?n-s%n:1,i=qe(t,i)}return r===!0?-1:t.length}const Mh="ͼ",gm=typeof Symbol>"u"?"__"+Mh:Symbol.for(Mh),Ih=typeof Symbol>"u"?"__styleSet"+Math.floor(Math.random()*1e8):Symbol("styleSet"),vm=typeof globalThis<"u"?globalThis:typeof window<"u"?window:{};class hr{constructor(e,n){this.rules=[];let{finish:r}=n||{};function i(o){return/^@/.test(o)?[o]:o.split(/,\s*/)}function s(o,l,a,c){let u=[],h=/^@(\w+)\b/.exec(o[0]),f=h&&h[1]=="keyframes";if(h&&l==null)return a.push(o[0]+";");for(let d in l){let p=l[d];if(/&/.test(d))s(d.split(/,\s*/).map(m=>o.map(w=>m.replace(/&/,w))).reduce((m,w)=>m.concat(w)),p,a);else if(p&&typeof p=="object"){if(!h)throw new RangeError("The value of a property ("+d+") should be a primitive value.");s(i(d),p,u,f)}else p!=null&&u.push(d.replace(/_.*/,"").replace(/[A-Z]/g,m=>"-"+m.toLowerCase())+": "+p+";")}(u.length||f)&&a.push((r&&!h&&!c?o.map(r):o).join(", ")+" {"+u.join(" ")+"}")}for(let o in e)s(i(o),e[o],this.rules)}getRules(){return this.rules.join(`
`)}static newName(){let e=vm[gm]||1;return vm[gm]=e+1,Mh+e.toString(36)}static mount(e,n,r){let i=e[Ih],s=r&&r.nonce;i?s&&i.setNonce(s):i=new Ob(e,s),i.mount(Array.isArray(n)?n:[n],e)}}let ym=new Map;class Ob{constructor(e,n){let r=e.ownerDocument||e,i=r.defaultView;if(!e.head&&e.adoptedStyleSheets&&i.CSSStyleSheet){let s=ym.get(r);if(s)return e[Ih]=s;this.sheet=new i.CSSStyleSheet,ym.set(r,this)}else this.styleTag=r.createElement("style"),n&&this.styleTag.setAttribute("nonce",n);this.modules=[],e[Ih]=this}mount(e,n){let r=this.sheet,i=0,s=0,o=!1;for(let l=0;l<e.length;l++){let a=e[l],c=this.modules.indexOf(a);if(c<s&&c>-1&&(this.modules.splice(c,1),o=!0,s--,c=-1),c==-1){if(this.modules.splice(s++,0,a),o=!0,r)for(let u=0;u<a.rules.length;u++)r.insertRule(a.rules[u],i++)}else{for(;s<c;)i+=this.modules[s++].rules.length;i+=a.rules.length,s++}}if(r)n.adoptedStyleSheets.indexOf(this.sheet)<0&&(n.adoptedStyleSheets=[this.sheet,...n.adoptedStyleSheets]);else{if(o){let a="";for(let c=0;c<this.modules.length;c++)a+=this.modules[c].getRules()+`
`;this.styleTag.textContent=a}let l=n.head||n;this.styleTag.parentNode!=l&&l.insertBefore(this.styleTag,l.firstChild)}}setNonce(e){this.styleTag&&this.styleTag.getAttribute("nonce")!=e&&this.styleTag.setAttribute("nonce",e)}}var fr={8:"Backspace",9:"Tab",10:"Enter",12:"NumLock",13:"Enter",16:"Shift",17:"Control",18:"Alt",20:"CapsLock",27:"Escape",32:" ",33:"PageUp",34:"PageDown",35:"End",36:"Home",37:"ArrowLeft",38:"ArrowUp",39:"ArrowRight",40:"ArrowDown",44:"PrintScreen",45:"Insert",46:"Delete",59:";",61:"=",91:"Meta",92:"Meta",106:"*",107:"+",108:",",109:"-",110:".",111:"/",144:"NumLock",145:"ScrollLock",160:"Shift",161:"Shift",162:"Control",163:"Control",164:"Alt",165:"Alt",173:"-",186:";",187:"=",188:",",189:"-",190:".",191:"/",192:"`",219:"[",220:"\\",221:"]",222:"'"},ao={48:")",49:"!",50:"@",51:"#",52:"$",53:"%",54:"^",55:"&",56:"*",57:"(",59:":",61:"+",173:"_",186:":",187:"+",188:"<",189:"_",190:">",191:"?",192:"~",219:"{",220:"|",221:"}",222:'"'},Nb=typeof navigator<"u"&&/Mac/.test(navigator.platform),Bb=typeof navigator<"u"&&/MSIE \d|Trident\/(?:[7-9]|\d{2,})\..*rv:(\d+)/.exec(navigator.userAgent);for(var Ue=0;Ue<10;Ue++)fr[48+Ue]=fr[96+Ue]=String(Ue);for(var Ue=1;Ue<=24;Ue++)fr[Ue+111]="F"+Ue;for(var Ue=65;Ue<=90;Ue++)fr[Ue]=String.fromCharCode(Ue+32),ao[Ue]=String.fromCharCode(Ue);for(var $c in fr)ao.hasOwnProperty($c)||(ao[$c]=fr[$c]);function _b(t){var e=Nb&&t.metaKey&&t.shiftKey&&!t.ctrlKey&&!t.altKey||Bb&&t.shiftKey&&t.key&&t.key.length==1||t.key=="Unidentified",n=!e&&t.key||(t.shiftKey?ao:fr)[t.keyCode]||t.key||"Unidentified";return n=="Esc"&&(n="Escape"),n=="Del"&&(n="Delete"),n=="Left"&&(n="ArrowLeft"),n=="Up"&&(n="ArrowUp"),n=="Right"&&(n="ArrowRight"),n=="Down"&&(n="ArrowDown"),n}function le(){var t=arguments[0];typeof t=="string"&&(t=document.createElement(t));var e=1,n=arguments[1];if(n&&typeof n=="object"&&n.nodeType==null&&!Array.isArray(n)){for(var r in n)if(Object.prototype.hasOwnProperty.call(n,r)){var i=n[r];typeof i=="string"?t.setAttribute(r,i):i!=null&&(t[r]=i)}e++}for(;e<arguments.length;e++)My(t,arguments[e]);return t}function My(t,e){if(typeof e=="string")t.appendChild(document.createTextNode(e));else if(e!=null)if(e.nodeType!=null)t.appendChild(e);else if(Array.isArray(e))for(var n=0;n<e.length;n++)My(t,e[n]);else throw new RangeError("Unsupported child node: "+e)}let rt=typeof navigator<"u"?navigator:{userAgent:"",vendor:"",platform:""},Ah=typeof document<"u"?document:{documentElement:{style:{}}};const Ph=/Edge\/(\d+)/.exec(rt.userAgent),Iy=/MSIE \d/.test(rt.userAgent),Eh=/Trident\/(?:[7-9]|\d{2,})\..*rv:(\d+)/.exec(rt.userAgent),Ja=!!(Iy||Eh||Ph),wm=!Ja&&/gecko\/(\d+)/i.test(rt.userAgent),Gc=!Ja&&/Chrome\/(\d+)/.exec(rt.userAgent),Sm="webkitFontSmoothing"in Ah.documentElement.style,Lh=!Ja&&/Apple Computer/.test(rt.vendor),km=Lh&&(/Mobile\/\w+/.test(rt.userAgent)||rt.maxTouchPoints>2);var _={mac:km||/Mac/.test(rt.platform),windows:/Win/.test(rt.platform),linux:/Linux|X11/.test(rt.platform),ie:Ja,ie_version:Iy?Ah.documentMode||6:Eh?+Eh[1]:Ph?+Ph[1]:0,gecko:wm,gecko_version:wm?+(/Firefox\/(\d+)/.exec(rt.userAgent)||[0,0])[1]:0,chrome:!!Gc,chrome_version:Gc?+Gc[1]:0,ios:km,android:/Android\b/.test(rt.userAgent),webkit:Sm,webkit_version:Sm?+(/\bAppleWebKit\/(\d+)/.exec(rt.userAgent)||[0,0])[1]:0,safari:Lh,safari_version:Lh?+(/\bVersion\/(\d+(\.\d+)?)/.exec(rt.userAgent)||[0,0])[1]:0,tabSize:Ah.documentElement.style.tabSize!=null?"tab-size":"-moz-tab-size"};function ud(t,e){for(let n in t)n=="class"&&e.class?e.class+=" "+t.class:n=="style"&&e.style?e.style+=";"+t.style:e[n]=t[n];return e}const ma=Object.create(null);function hd(t,e,n){if(t==e)return!0;t||(t=ma),e||(e=ma);let r=Object.keys(t),i=Object.keys(e);if(r.length-0!=i.length-0)return!1;for(let s of r)if(s!=n&&(i.indexOf(s)==-1||t[s]!==e[s]))return!1;return!0}function qb(t,e){for(let n=t.attributes.length-1;n>=0;n--){let r=t.attributes[n].name;e[r]==null&&t.removeAttribute(r)}for(let n in e){let r=e[n];n=="style"?t.style.cssText=r:t.getAttribute(n)!=r&&t.setAttribute(n,r)}}function xm(t,e,n){let r=!1;if(e)for(let i in e)n&&i in n||(r=!0,i=="style"?t.style.cssText="":t.removeAttribute(i));if(n)for(let i in n)e&&e[i]==n[i]||(r=!0,i=="style"?t.style.cssText=n[i]:t.setAttribute(i,n[i]));return r}function Wb(t){let e=Object.create(null);for(let n=0;n<t.attributes.length;n++){let r=t.attributes[n];e[r.name]=r.value}return e}class qn{eq(e){return!1}updateDOM(e,n,r){return!1}compare(e){return this==e||this.constructor==e.constructor&&this.eq(e)}get estimatedHeight(){return-1}get lineBreaks(){return 0}ignoreEvent(e){return!0}coordsAt(e,n,r){return null}get isHidden(){return!1}get editable(){return!1}destroy(e){}}var Ge=function(t){return t[t.Text=0]="Text",t[t.WidgetBefore=1]="WidgetBefore",t[t.WidgetAfter=2]="WidgetAfter",t[t.WidgetRange=3]="WidgetRange",t}(Ge||(Ge={}));class U extends ur{constructor(e,n,r,i){super(),this.startSide=e,this.endSide=n,this.widget=r,this.spec=i}get heightRelevant(){return!1}static mark(e){return new Mo(e)}static widget(e){let n=Math.max(-1e4,Math.min(1e4,e.side||0)),r=!!e.block;return n+=r&&!e.inlineOrder?n>0?3e8:-4e8:n>0?1e8:-1e8,new Ur(e,n,n,r,e.widget||null,!1)}static replace(e){let n=!!e.block,r,i;if(e.isBlockGap)r=-5e8,i=4e8;else{let{start:s,end:o}=Ay(e,n);r=(s?n?-3e8:-1:5e8)-1,i=(o?n?2e8:1:-6e8)+1}return new Ur(e,r,i,n,e.widget||null,!0)}static line(e){return new Io(e)}static set(e,n=!1){return X.of(e,n)}hasHeight(){return this.widget?this.widget.estimatedHeight>-1:!1}}U.none=X.empty;class Mo extends U{constructor(e){let{start:n,end:r}=Ay(e);super(n?-1:5e8,r?1:-6e8,null,e),this.tagName=e.tagName||"span",this.attrs=e.class&&e.attributes?ud(e.attributes,{class:e.class}):e.class?{class:e.class}:e.attributes||ma}eq(e){return this==e||e instanceof Mo&&this.tagName==e.tagName&&hd(this.attrs,e.attrs)}range(e,n=e){if(e>=n)throw new RangeError("Mark decorations may not be empty");return super.range(e,n)}}Mo.prototype.point=!1;class Io extends U{constructor(e){super(-2e8,-2e8,null,e)}eq(e){return e instanceof Io&&this.spec.class==e.spec.class&&hd(this.spec.attributes,e.spec.attributes)}range(e,n=e){if(n!=e)throw new RangeError("Line decoration ranges must be zero-length");return super.range(e,n)}}Io.prototype.mapMode=it.TrackBefore;Io.prototype.point=!0;class Ur extends U{constructor(e,n,r,i,s,o){super(n,r,s,e),this.block=i,this.isReplace=o,this.mapMode=i?n<=0?it.TrackBefore:it.TrackAfter:it.TrackDel}get type(){return this.startSide!=this.endSide?Ge.WidgetRange:this.startSide<=0?Ge.WidgetBefore:Ge.WidgetAfter}get heightRelevant(){return this.block||!!this.widget&&(this.widget.estimatedHeight>=5||this.widget.lineBreaks>0)}eq(e){return e instanceof Ur&&Fb(this.widget,e.widget)&&this.block==e.block&&this.startSide==e.startSide&&this.endSide==e.endSide}range(e,n=e){if(this.isReplace&&(e>n||e==n&&this.startSide>0&&this.endSide<=0))throw new RangeError("Invalid range for replacement decoration");if(!this.isReplace&&n!=e)throw new RangeError("Widget decorations can only have zero-length ranges");return super.range(e,n)}}Ur.prototype.point=!0;function Ay(t,e=!1){let{inclusiveStart:n,inclusiveEnd:r}=t;return n==null&&(n=t.inclusive),r==null&&(r=t.inclusive),{start:n??e,end:r??e}}function Fb(t,e){return t==e||!!(t&&e&&t.compare(e))}function Ai(t,e,n,r=0){let i=n.length-1;i>=0&&n[i]+r>=t?n[i]=Math.max(n[i],e):n.push(t,e)}class co extends ur{constructor(e,n,r){super(),this.tagName=e,this.attributes=n,this.rank=r}eq(e){return e==this||e instanceof co&&this.tagName==e.tagName&&hd(this.attributes,e.attributes)}static create(e){return new co(e.tagName,e.attributes||ma,e.rank==null?50:Math.max(0,Math.min(e.rank,100)))}static set(e,n=!1){return X.of(e,n)}}co.prototype.startSide=co.prototype.endSide=-1;const Fe=t=>t.nodeType==1,Vi=t=>t.nodeType==3;function uo(t){let e;return t.nodeType==11?e=t.getSelection?t:t.ownerDocument:e=t,e.getSelection()}function Dh(t,e){return e?t==e||t.contains(e.nodeType!=1?e.parentNode:e):!1}function _s(t,e){if(!e.anchorNode)return!1;try{return Dh(t,e.anchorNode)}catch{return!1}}function Nl(t){return Vi(t)?ho(t,0,t.nodeValue.length).getClientRects():Fe(t)?t.getClientRects():[]}function qs(t,e,n,r){return n?Cm(t,e,n,r,-1)||Cm(t,e,n,r,1):!1}function dr(t){for(var e=0;;e++)if(t=t.previousSibling,!t)return e}function ga(t){return Fe(t)&&/^(DIV|P|LI|UL|OL|BLOCKQUOTE|DD|DT|H\d|SECTION|PRE)$/.test(t.nodeName)}function Cm(t,e,n,r,i){for(;;){if(t==n&&e==r)return!0;if(e==(i<0?0:Nn(t))){if(t.nodeName=="DIV")return!1;let s=t.parentNode;if(!s||!Fe(s))return!1;e=dr(t)+(i<0?0:1),t=s}else if(Fe(t)){if(t=t.childNodes[e+(i<0?-1:0)],Fe(t)&&(t.nodeName=="IMG"||t.nodeName=="BR"||t.contentEditable=="false"))return!1;e=i<0?Nn(t):0}else return!1}}function Nn(t){return Vi(t)?t.nodeValue.length:t.childNodes.length}function va(t,e){let{left:n,right:r}=t;if(n==r)return t;let i=e?n:r;return{left:i,right:i,top:t.top,bottom:t.bottom}}function zb(t){let e=t.visualViewport;return e?{left:0,right:e.width,top:0,bottom:e.height}:{left:0,right:t.innerWidth,top:0,bottom:t.innerHeight}}function Py(t,e){let n=e.width/t.offsetWidth,r=e.height/t.offsetHeight;return(n>.995&&n<1.005||!isFinite(n)||Math.abs(e.width-t.offsetWidth)<1)&&(n=1),(r>.995&&r<1.005||!isFinite(r)||Math.abs(e.height-t.offsetHeight)<1)&&(r=1),{scaleX:n,scaleY:r}}function jb(t,e,n,r,i,s,o,l){let a=t.ownerDocument,c=a.defaultView||window;for(let u=t,h=!1;u&&!h;)if(Fe(u)){let f,d=u==a.body,p=1,m=1;if(d)f=zb(c);else{if(/^(fixed|sticky)$/.test(getComputedStyle(u).position)&&(h=!0),u.scrollHeight<=u.clientHeight&&u.scrollWidth<=u.clientWidth){u=u.assignedSlot||u.parentNode;continue}let v=u.getBoundingClientRect();({scaleX:p,scaleY:m}=Py(u,v)),f={left:v.left,right:v.left+u.clientWidth*p,top:v.top,bottom:v.top+u.clientHeight*m}}let w=0,g=0;if(i=="nearest")e.top<f.top+o?(g=e.top-(f.top+o),n>0&&e.bottom>f.bottom+g&&(g=e.bottom-f.bottom+o)):e.bottom>f.bottom-o&&(g=e.bottom-f.bottom+o,n<0&&e.top-g<f.top&&(g=e.top-(f.top+o)));else{let v=e.bottom-e.top,y=f.bottom-f.top;g=(i=="center"&&v<=y?e.top+v/2-y/2:i=="start"||i=="center"&&n<0?e.top-o:e.bottom-y+o)-f.top}if(r=="nearest"?e.left<f.left+s?(w=e.left-(f.left+s),n>0&&e.right>f.right+w&&(w=e.right-f.right+s)):e.right>f.right-s&&(w=e.right-f.right+s,n<0&&e.left<f.left+w&&(w=e.left-(f.left+s))):w=(r=="center"?e.left+(e.right-e.left)/2-(f.right-f.left)/2:r=="start"==l?e.left-s:e.right-(f.right-f.left)+s)-f.left,w||g)if(d)c.scrollBy(w,g);else{let v=0,y=0;if(g){let x=u.scrollTop;u.scrollTop+=g/m,y=(u.scrollTop-x)*m}if(w){let x=u.scrollLeft;u.scrollLeft+=w/p,v=(u.scrollLeft-x)*p}e={left:e.left-v,top:e.top-y,right:e.right-v,bottom:e.bottom-y},v&&Math.abs(v-w)<1&&(r="nearest"),y&&Math.abs(y-g)<1&&(i="nearest")}if(d)break;(e.top<f.top||e.bottom>f.bottom||e.left<f.left||e.right>f.right)&&(e={left:Math.max(e.left,f.left),right:Math.min(e.right,f.right),top:Math.max(e.top,f.top),bottom:Math.min(e.bottom,f.bottom)}),u=u.assignedSlot||u.parentNode}else if(u.nodeType==11)u=u.host;else break}function Ey(t,e=!0){let n=t.ownerDocument,r=null,i=null;for(let s=t.parentNode;s&&!(s==n.body||(!e||r)&&i);)if(Fe(s))!i&&s.scrollHeight>s.clientHeight&&(i=s),e&&!r&&s.scrollWidth>s.clientWidth&&(r=s),s=s.assignedSlot||s.parentNode;else if(s.nodeType==11)s=s.host;else break;return{x:r,y:i}}class Vb{constructor(){this.anchorNode=null,this.anchorOffset=0,this.focusNode=null,this.focusOffset=0}eq(e){return this.anchorNode==e.anchorNode&&this.anchorOffset==e.anchorOffset&&this.focusNode==e.focusNode&&this.focusOffset==e.focusOffset}setRange(e){let{anchorNode:n,focusNode:r}=e;this.set(n,Math.min(e.anchorOffset,n?Nn(n):0),r,Math.min(e.focusOffset,r?Nn(r):0))}set(e,n,r,i){this.anchorNode=e,this.anchorOffset=n,this.focusNode=r,this.focusOffset=i}}function Ly(t){let e=[];for(let n=t;n;n=n.nodeType==11?n.host:n.parentNode)Fe(n)&&e.push({node:n,left:n.scrollLeft,top:n.scrollTop});return e}function Dy(t,e=!0){for(let{node:n,left:r,top:i}of t)e&&n.scrollTop!=i&&(n.scrollTop=i),n.scrollLeft!=r&&(n.scrollLeft=r)}let Mr=null;_.safari&&_.safari_version>=26&&(Mr=!1);function Oy(t){if(t.setActive)return t.setActive();if(Mr)return t.focus(Mr);let e=Ly(t);t.focus(Mr==null?{get preventScroll(){return Mr={preventScroll:!0},!0}}:void 0),Mr||(Mr=!1,Dy(e))}let bm;function ho(t,e,n=e){let r=bm||(bm=document.createRange());return r.setEnd(t,n),r.setStart(t,e),r}function Pi(t,e,n,r){let i={key:e,code:e,keyCode:n,which:n,cancelable:!0};r&&({altKey:i.altKey,ctrlKey:i.ctrlKey,shiftKey:i.shiftKey,metaKey:i.metaKey}=r);let s=new KeyboardEvent("keydown",i);s.synthetic=!0,t.dispatchEvent(s);let o=new KeyboardEvent("keyup",i);return o.synthetic=!0,t.dispatchEvent(o),s.defaultPrevented||o.defaultPrevented}function Hb(t){for(;t;){if(t&&(t.nodeType==9||t.nodeType==11&&t.host))return t;t=t.assignedSlot||t.parentNode}return null}function Ub(t,e){let n=e.focusNode,r=e.focusOffset;if(!n||e.anchorNode!=n||e.anchorOffset!=r)return!1;for(r=Math.min(r,Nn(n));;)if(r){if(!Fe(n))return!1;let i=n.childNodes[r-1];i.contentEditable=="false"?r--:(n=i,r=Nn(n))}else{if(n==t)return!0;r=dr(n),n=n.parentNode}}function Ny(t){return t instanceof Window?t.pageYOffset>Math.max(0,t.document.documentElement.scrollHeight-t.innerHeight-4):t.scrollTop>Math.max(1,t.scrollHeight-t.clientHeight-4)}function By(t,e){for(let n=t,r=e;;){if(Vi(n)&&r>0)return{node:n,offset:r};if(Fe(n)&&r>0){if(n.contentEditable=="false")return null;n=n.childNodes[r-1],r=Nn(n)}else if(n.parentNode&&!ga(n))r=dr(n),n=n.parentNode;else return null}}function _y(t,e){for(let n=t,r=e;;){if(Vi(n)&&r<n.nodeValue.length)return{node:n,offset:r};if(Fe(n)&&r<n.childNodes.length){if(n.contentEditable=="false")return null;n=n.childNodes[r],r=0}else if(n.parentNode&&!ga(n))r=dr(n)+1,n=n.parentNode;else return null}}class Gt{constructor(e,n,r=!0){this.node=e,this.offset=n,this.precise=r}static before(e,n){return new Gt(e.parentNode,dr(e),n)}static after(e,n){return new Gt(e.parentNode,dr(e)+1,n)}}var ue=function(t){return t[t.LTR=0]="LTR",t[t.RTL=1]="RTL",t}(ue||(ue={}));const $r=ue.LTR,fd=ue.RTL;function qy(t){let e=[];for(let n=0;n<t.length;n++)e.push(1<<+t[n]);return e}const $b=qy("88888888888888888888888888888888888666888888787833333333337888888000000000000000000000000008888880000000000000000000000000088888888888888888888888888888888888887866668888088888663380888308888800000000000000000000000800000000000000000000000000000008"),Gb=qy("4444448826627288999999999992222222222222222222222222222222222222222222222229999999999999999999994444444444644222822222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222999999949999999229989999223333333333"),Oh=Object.create(null),sn=[];for(let t of["()","[]","{}"]){let e=t.charCodeAt(0),n=t.charCodeAt(1);Oh[e]=n,Oh[n]=-e}function Wy(t){return t<=247?$b[t]:1424<=t&&t<=1524?2:1536<=t&&t<=1785?Gb[t-1536]:1774<=t&&t<=2220?4:8192<=t&&t<=8204?256:64336<=t&&t<=65023?4:1}const Kb=/[\u0590-\u05f4\u0600-\u06ff\u0700-\u08ac\ufb50-\ufdff]/;class pn{get dir(){return this.level%2?fd:$r}constructor(e,n,r){this.from=e,this.to=n,this.level=r}side(e,n){return this.dir==n==e?this.to:this.from}forward(e,n){return e==(this.dir==n)}static find(e,n,r,i){let s=-1;for(let o=0;o<e.length;o++){let l=e[o];if(l.from<=n&&l.to>=n){if(l.level==r)return o;(s<0||(i!=0?i<0?l.from<n:l.to>n:e[s].level>l.level))&&(s=o)}}if(s<0)throw new RangeError("Index out of range");return s}}function Fy(t,e){if(t.length!=e.length)return!1;for(let n=0;n<t.length;n++){let r=t[n],i=e[n];if(r.from!=i.from||r.to!=i.to||r.direction!=i.direction||!Fy(r.inner,i.inner))return!1}return!0}const ce=[];function Qb(t,e,n,r,i){for(let s=0;s<=r.length;s++){let o=s?r[s-1].to:e,l=s<r.length?r[s].from:n,a=s?256:i;for(let c=o,u=a,h=a;c<l;c++){let f=Wy(t.charCodeAt(c));f==512?f=u:f==8&&h==4&&(f=16),ce[c]=f==4?2:f,f&7&&(h=f),u=f}for(let c=o,u=a,h=a;c<l;c++){let f=ce[c];if(f==128)c<l-1&&u==ce[c+1]&&u&24?f=ce[c]=u:ce[c]=256;else if(f==64){let d=c+1;for(;d<l&&ce[d]==64;)d++;let p=c&&u==8||d<n&&ce[d]==8?h==1?1:8:256;for(let m=c;m<d;m++)ce[m]=p;c=d-1}else f==8&&h==1&&(ce[c]=1);u=f,f&7&&(h=f)}}}function Yb(t,e,n,r,i){let s=i==1?2:1;for(let o=0,l=0,a=0;o<=r.length;o++){let c=o?r[o-1].to:e,u=o<r.length?r[o].from:n;for(let h=c,f,d,p;h<u;h++)if(d=Oh[f=t.charCodeAt(h)])if(d<0){for(let m=l-3;m>=0;m-=3)if(sn[m+1]==-d){let w=sn[m+2],g=w&2?i:w&4?w&1?s:i:0;g&&(ce[h]=ce[sn[m]]=g),l=m;break}}else{if(sn.length==189)break;sn[l++]=h,sn[l++]=f,sn[l++]=a}else if((p=ce[h])==2||p==1){let m=p==i;a=m?0:1;for(let w=l-3;w>=0;w-=3){let g=sn[w+2];if(g&2)break;if(m)sn[w+2]|=2;else{if(g&4)break;sn[w+2]|=4}}}}}function Xb(t,e,n,r){for(let i=0,s=r;i<=n.length;i++){let o=i?n[i-1].to:t,l=i<n.length?n[i].from:e;for(let a=o;a<l;){let c=ce[a];if(c==256){let u=a+1;for(;;)if(u==l){if(i==n.length)break;u=n[i++].to,l=i<n.length?n[i].from:e}else if(ce[u]==256)u++;else break;let h=s==1,f=(u<e?ce[u]:r)==1,d=h==f?h?1:2:r;for(let p=u,m=i,w=m?n[m-1].to:t;p>a;)p==w&&(p=n[--m].from,w=m?n[m-1].to:t),ce[--p]=d;a=u}else s=c,a++}}}function Nh(t,e,n,r,i,s,o){let l=r%2?2:1;if(r%2==i%2)for(let a=e,c=0;a<n;){let u=!0,h=!1;if(c==s.length||a<s[c].from){let m=ce[a];m!=l&&(u=!1,h=m==16)}let f=!u&&l==1?[]:null,d=u?r:r+1,p=a;e:for(;;)if(c<s.length&&p==s[c].from){if(h)break e;let m=s[c];if(!u)for(let w=m.to,g=c+1;;){if(w==n)break e;if(g<s.length&&s[g].from==w)w=s[g++].to;else{if(ce[w]==l)break e;break}}if(c++,f)f.push(m);else{m.from>a&&o.push(new pn(a,m.from,d));let w=m.direction==$r!=!(d%2);Bh(t,w?r+1:r,i,m.inner,m.from,m.to,o),a=m.to}p=m.to}else{if(p==n||(u?ce[p]!=l:ce[p]==l))break;p++}f?Nh(t,a,p,r+1,i,f,o):a<p&&o.push(new pn(a,p,d)),a=p}else for(let a=n,c=s.length;a>e;){let u=!0,h=!1;if(!c||a>s[c-1].to){let m=ce[a-1];m!=l&&(u=!1,h=m==16)}let f=!u&&l==1?[]:null,d=u?r:r+1,p=a;e:for(;;)if(c&&p==s[c-1].to){if(h)break e;let m=s[--c];if(!u)for(let w=m.from,g=c;;){if(w==e)break e;if(g&&s[g-1].to==w)w=s[--g].from;else{if(ce[w-1]==l)break e;break}}if(f)f.push(m);else{m.to<a&&o.push(new pn(m.to,a,d));let w=m.direction==$r!=!(d%2);Bh(t,w?r+1:r,i,m.inner,m.from,m.to,o),a=m.from}p=m.from}else{if(p==e||(u?ce[p-1]!=l:ce[p-1]==l))break;p--}f?Nh(t,p,a,r+1,i,f,o):p<a&&o.push(new pn(p,a,d)),a=p}}function Bh(t,e,n,r,i,s,o){let l=e%2?2:1;Qb(t,i,s,r,l),Yb(t,i,s,r,l),Xb(i,s,r,l),Nh(t,i,s,e,n,r,o)}function Jb(t,e,n){if(!t)return[new pn(0,0,e==fd?1:0)];if(e==$r&&!n.length&&!Kb.test(t))return zy(t.length);if(n.length)for(;t.length>ce.length;)ce[ce.length]=256;let r=[],i=e==$r?0:1;return Bh(t,i,i,n,0,t.length,r),r}function zy(t){return[new pn(0,t,0)]}let jy="";function Zb(t,e,n,r,i){var s;if(!t.length)return null;let o=r.head-t.from,l;if(r.head==t.from&&r.assoc<0){if(!i)return null;o=e[l=0].side(!1,n)}else if(r.head==t.to&&r.assoc>0){if(i)return null;o=e[l=e.length-1].side(!0,n)}else l=pn.find(e,o,(s=r.bidiLevel)!==null&&s!==void 0?s:-1,r.assoc);let a=e[l],c=a.side(i,n);if(o==c){let f=l+=i?1:-1;if(f<0||f>=e.length)return null;a=e[l=f],o=a.side(!i,n),c=a.side(i,n)}let u=qe(t.text,o,a.forward(i,n));(u<a.from||u>a.to)&&(u=c),jy=t.text.slice(Math.min(o,u),Math.max(o,u));let h=l==(i?e.length-1:0)?null:e[l+(i?1:-1)];if(u==c){if(!h)return i?R.cursor(t.to,1):R.cursor(t.from,-1);if(h.level+(i?0:1)<a.level)return R.cursor(h.side(!i,n)+t.from,h.forward(i,n)?1:-1,h.level)}return R.cursor(u+t.from,a.forward(i,n)?-1:1,a.level)}function eT(t,e,n){for(let r=e;r<n;r++){let i=Wy(t.charCodeAt(r));if(i==1)return $r;if(i==2||i==4)return fd}return $r}const Vy=z.define(),Hy=z.define(),Uy=z.define(),$y=z.define(),_h=z.define(),Gy=z.define(),Ky=z.define(),dd=z.define(),pd=z.define(),Qy=z.define({combine:t=>t.some(e=>e)}),Yy=z.define({combine:t=>t.some(e=>e)}),Xy=z.define();class Ei{constructor(e,n,r,i,s,o=!1){this.range=e,this.y=n,this.x=r,this.yMargin=i,this.xMargin=s,this.isSnapshot=o}map(e){return e.empty?this:new Ei(this.range.map(e),this.y,this.x,this.yMargin,this.xMargin,this.isSnapshot)}clip(e){return this.range.to<=e.doc.length?this:new Ei(R.cursor(e.doc.length),this.y,this.x,this.yMargin,this.xMargin,this.isSnapshot)}}const tl=K.define({map:(t,e)=>t.map(e)}),Jy=K.define();function st(t,e,n){let r=t.facet($y);r.length?r[0](e):window.onerror&&window.onerror(String(e),n,void 0,void 0,e)||(n?console.error(n+":",e):console.error(e))}const Rn=z.define({combine:t=>t.length?t[0]:!0});let tT=0;const yi=z.define({combine(t){return t.filter((e,n)=>{for(let r=0;r<n;r++)if(t[r].plugin==e.plugin)return!1;return!0})}});class Me{constructor(e,n,r,i,s){this.id=e,this.create=n,this.domEventHandlers=r,this.domEventObservers=i,this.baseExtensions=s(this),this.extension=this.baseExtensions.concat(yi.of({plugin:this,arg:void 0}))}of(e){return this.baseExtensions.concat(yi.of({plugin:this,arg:e}))}static define(e,n){const{eventHandlers:r,eventObservers:i,provide:s,decorations:o}=n||{};return new Me(tT++,e,r,i,l=>{let a=[];return o&&a.push(Za.of(c=>{let u=c.plugin(l);return u?o(u):U.none})),s&&a.push(s(l)),a})}static fromClass(e,n){return Me.define((r,i)=>new e(r,i),n)}}class Kc{constructor(e){this.spec=e,this.mustUpdate=null,this.value=null}get plugin(){return this.spec&&this.spec.plugin}update(e){if(this.value){if(this.mustUpdate){let n=this.mustUpdate;if(this.mustUpdate=null,this.value.update)try{this.value.update(n)}catch(r){if(st(n.state,r,"CodeMirror plugin crashed"),this.value.destroy)try{this.value.destroy()}catch{}this.deactivate()}}}else if(this.spec)try{this.value=this.spec.plugin.create(e,this.spec.arg)}catch(n){st(e.state,n,"CodeMirror plugin crashed"),this.deactivate()}return this}destroy(e){var n;if(!((n=this.value)===null||n===void 0)&&n.destroy)try{this.value.destroy()}catch(r){st(e.state,r,"CodeMirror plugin crashed")}}deactivate(){this.spec=this.value=null}}const Zy=z.define(),md=z.define(),Za=z.define(),ew=z.define(),gd=z.define(),Ao=z.define(),tw=z.define();function Tm(t,e){let n=t.state.facet(tw);if(!n.length)return n;let r=n.map(s=>s instanceof Function?s(t):s),i=[];return X.spans(r,e.from,e.to,{point(){},span(s,o,l,a){let c=s-e.from,u=o-e.from,h=i;for(let f=l.length-1;f>=0;f--,a--){let d=l[f].spec.bidiIsolate,p;if(d==null&&(d=eT(e.text,c,u)),a>0&&h.length&&(p=h[h.length-1]).to==c&&p.direction==d)p.to=u,h=p.inner;else{let m={from:c,to:u,direction:d,inner:[]};h.push(m),h=m.inner}}}}),i}const nw=z.define();function vd(t){let e=0,n=0,r=0,i=0;for(let s of t.state.facet(nw)){let o=s(t);o&&(o.left!=null&&(e=Math.max(e,o.left)),o.right!=null&&(n=Math.max(n,o.right)),o.top!=null&&(r=Math.max(r,o.top)),o.bottom!=null&&(i=Math.max(i,o.bottom)))}return{left:e,right:n,top:r,bottom:i}}const Cs=z.define();class Ot{constructor(e,n,r,i){this.fromA=e,this.toA=n,this.fromB=r,this.toB=i}join(e){return new Ot(Math.min(this.fromA,e.fromA),Math.max(this.toA,e.toA),Math.min(this.fromB,e.fromB),Math.max(this.toB,e.toB))}addToSet(e){let n=e.length,r=this;for(;n>0;n--){let i=e[n-1];if(!(i.fromA>r.toA)){if(i.toA<r.fromA)break;r=r.join(i),e.splice(n-1,1)}}return e.splice(n,0,r),e}static extendWithRanges(e,n){if(n.length==0)return e;let r=[];for(let i=0,s=0,o=0;;){let l=i<e.length?e[i].fromB:1e9,a=s<n.length?n[s]:1e9,c=Math.min(l,a);if(c==1e9)break;let u=c+o,h=c,f=u;for(;;)if(s<n.length&&n[s]<=h){let d=n[s+1];s+=2,h=Math.max(h,d);for(let p=i;p<e.length&&e[p].fromB<=h;p++)o=e[p].toA-e[p].toB;f=Math.max(f,d+o)}else if(i<e.length&&e[i].fromB<=h){let d=e[i++];h=Math.max(h,d.toB),f=Math.max(f,d.toA),o=d.toA-d.toB}else break;r.push(new Ot(u,f,c,h))}return r}}class ya{constructor(e,n,r){this.view=e,this.state=n,this.transactions=r,this.flags=0,this.startState=e.state,this.changes=De.empty(this.startState.doc.length);for(let s of r)this.changes=this.changes.compose(s.changes);let i=[];this.changes.iterChangedRanges((s,o,l,a)=>i.push(new Ot(s,o,l,a))),this.changedRanges=i}static create(e,n,r){return new ya(e,n,r)}get viewportChanged(){return(this.flags&4)>0}get viewportMoved(){return(this.flags&8)>0}get heightChanged(){return(this.flags&2)>0}get geometryChanged(){return this.docChanged||(this.flags&18)>0}get focusChanged(){return(this.flags&1)>0}get docChanged(){return!this.changes.empty}get selectionSet(){return this.transactions.some(e=>e.selection)}get empty(){return this.flags==0&&this.transactions.length==0}}const nT=[];class Re{constructor(e,n,r=0){this.dom=e,this.length=n,this.flags=r,this.parent=null,e.cmTile=this}get breakAfter(){return this.flags&1}get children(){return nT}isWidget(){return!1}get isHidden(){return!1}isComposite(){return!1}isLine(){return!1}isText(){return!1}isBlock(){return!1}get domAttrs(){return null}sync(e){if(this.flags|=2,this.flags&4){this.flags&=-5;let n=this.domAttrs;n&&qb(this.dom,n)}}toString(){return this.constructor.name+(this.children.length?`(${this.children})`:"")+(this.breakAfter?"#":"")}destroy(){this.parent=null}setDOM(e){this.dom=e,e.cmTile=this}get posAtStart(){return this.parent?this.parent.posBefore(this):0}get posAtEnd(){return this.posAtStart+this.length}posBefore(e,n=this.posAtStart){let r=n;for(let i of this.children){if(i==e)return r;r+=i.length+i.breakAfter}throw new RangeError("Invalid child in posBefore")}posAfter(e){return this.posBefore(e)+e.length}covers(e){return!0}coordsIn(e,n,r){return null}domPosFor(e,n){let r=dr(this.dom),i=this.length?e>0:n>0;return new Gt(this.parent.dom,r+(i?1:0),e==0||e==this.length)}markDirty(e){this.flags&=-3,e&&(this.flags|=4),this.parent&&this.parent.flags&2&&this.parent.markDirty(!1)}get overrideDOMText(){return null}get root(){for(let e=this;e;e=e.parent)if(e instanceof tc)return e;return null}static get(e){return e.cmTile}}class ec extends Re{constructor(e){super(e,0),this._children=[]}isComposite(){return!0}get children(){return this._children}get lastChild(){return this.children.length?this.children[this.children.length-1]:null}append(e){this.children.push(e),e.parent=this}sync(e){if(this.flags&2)return;super.sync(e);let n=this.dom,r=null,i,s=(e==null?void 0:e.node)==n?e:null,o=0;for(let l of this.children){if(l.sync(e),o+=l.length+l.breakAfter,i=r?r.nextSibling:n.firstChild,s&&i!=l.dom&&(s.written=!0),l.dom.parentNode==n)for(;i&&i!=l.dom;)i=Rm(i);else n.insertBefore(l.dom,i);r=l.dom}for(i=r?r.nextSibling:n.firstChild,s&&i&&(s.written=!0);i;)i=Rm(i);this.length=o}}function Rm(t){let e=t.nextSibling;return t.parentNode.removeChild(t),e}class tc extends ec{constructor(e,n){super(n),this.view=e}owns(e){for(;e;e=e.parent)if(e==this)return!0;return!1}isBlock(){return!0}nearest(e){for(;;){if(!e)return null;let n=Re.get(e);if(n&&this.owns(n))return n;e=e.parentNode}}blockTiles(e){for(let n=[],r=this,i=0,s=0;;)if(i==r.children.length){if(!n.length)return;r=r.parent,r.breakAfter&&s++,i=n.pop()}else{let o=r.children[i++];if(o instanceof Pn)n.push(i),r=o,i=0;else{let l=s+o.length,a=e(o,s);if(a!==void 0)return a;s=l+o.breakAfter}}}resolveBlock(e,n){let r,i=-1,s,o=-1;if(this.blockTiles((l,a)=>{let c=a+l.length;if(e>=a&&e<=c){if(l.isWidget()&&n>=-1&&n<=1){if(l.flags&32)return!0;l.flags&16&&(r=void 0)}(a<e||e==c&&(n<-1?l.length:l.covers(1)))&&(!r||!l.isWidget()&&r.isWidget())&&(r=l,i=e-a),(c>e||e==a&&(n>1?l.length:l.covers(-1)))&&(!s||!l.isWidget()&&s.isWidget())&&(s=l,o=e-a)}}),!r&&!s)throw new Error("No tile at position "+e);return r&&n<0||!s?{tile:r,offset:i}:{tile:s,offset:o}}}class Pn extends ec{constructor(e,n){super(e),this.wrapper=n}isBlock(){return!0}covers(e){return this.children.length?e<0?this.children[0].covers(-1):this.lastChild.covers(1):!1}get domAttrs(){return this.wrapper.attributes}static of(e,n){let r=new Pn(n||document.createElement(e.tagName),e);return n||(r.flags|=4),r}}class Hi extends ec{constructor(e,n){super(e),this.attrs=n}isLine(){return!0}static start(e,n,r){let i=new Hi(n||document.createElement("div"),e);return(!n||!r)&&(i.flags|=4),i}get domAttrs(){return this.attrs}resolveInline(e,n,r=!1){let i=null,s=-1,o=null,l=-1;function a(u,h){for(let f=0,d=0;f<u.children.length&&d<=h;f++){let p=u.children[f],m=d+p.length;m>=h&&(p.isComposite()?a(p,h-d):(!o||r&&o.isHidden&&(n>0&&!(o.flags&32)||r&&iT(o,p)))&&(m>h||p.flags&32&&n<=1)?(o=p,l=h-d):(d<h||p.flags&16&&!(r&&p.isHidden)&&n>=-1)&&(i=p,s=h-d)),d=m}}a(this,e);let c=(n<0?i:o)||i||o;return c?{tile:c,offset:c==i?s:l}:null}coordsIn(e,n,r){let i=this.resolveInline(e,n,!0);return i?i.tile.coordsIn(Math.max(0,i.offset),n,r):rT(this)}domIn(e,n){let r=this.resolveInline(e,n);if(r){let{tile:i,offset:s}=r;if(this.dom.contains(i.dom))return i.isText()?new Gt(i.dom,Math.min(i.dom.nodeValue.length,s)):i.domPosFor(s,i.flags&16?1:i.flags&32?-1:n);let o=r.tile.parent,l=!1;for(let a of o.children){if(l)return new Gt(a.dom,0);a==r.tile&&(l=!0)}}return new Gt(this.dom,0)}}function rT(t){let e=t.dom.lastChild;if(!e)return t.dom.getBoundingClientRect();let n=Nl(e);return n[n.length-1]||null}function iT(t,e){let n=t.coordsIn(0,1),r=e.coordsIn(0,1);return n&&r&&r.top<n.bottom}class pt extends ec{constructor(e,n){super(e),this.mark=n}get domAttrs(){return this.mark.attrs}static of(e,n){let r=new pt(n||document.createElement(e.tagName),e);return n||(r.flags|=4),r}}class Dr extends Re{constructor(e,n){super(e,n.length),this.text=n}sync(e){this.flags&2||(super.sync(e),this.dom.nodeValue!=this.text&&(e&&e.node==this.dom&&(e.written=!0),this.dom.nodeValue=this.text))}isText(){return!0}toString(){return JSON.stringify(this.text)}coordsIn(e,n,r){let i=this.dom.nodeValue.length;e>i&&(e=i);let s=e,o=e,l=0;e==0&&n<0||e==i&&n>=0?_.chrome||_.gecko||(e?(s--,l=1):o<i&&(o++,l=-1)):n<0?s--:o<i&&o++;let a=ho(this.dom,s,o).getClientRects();if(!a.length)return null;let c=a[(l?l<0:n>=0)?0:a.length-1];return _.safari&&!l&&c.width==0&&(c=Array.prototype.find.call(a,u=>u.width)||c),r==null?c:va(c,(l?l>0:n<0)==r)}static of(e,n){let r=new Dr(n||document.createTextNode(e),e);return n||(r.flags|=2),r}}class Gr extends Re{constructor(e,n,r,i){super(e,n,i),this.widget=r}isWidget(){return!0}get isHidden(){return this.widget.isHidden}covers(e){return this.flags&48?!1:(this.flags&(e<0?64:128))>0}coordsIn(e,n){return this.coordsInWidget(e,n,!1)}coordsInWidget(e,n,r){let i=this.widget.coordsAt(this.dom,e,n);if(i)return i;if(r)return va(this.dom.getBoundingClientRect(),this.length?e==0:n<=0);{let s=this.dom.getClientRects(),o=null;if(!s.length)return null;let l=this.flags&16?!0:this.flags&32?!1:e>0;for(let a=l?s.length-1:0;o=s[a],!(e>0?a==0:a==s.length-1||o.top<o.bottom);a+=l?-1:1);return va(o,!l)}}get overrideDOMText(){if(!this.length)return te.empty;let{root:e}=this;if(!e)return te.empty;let n=this.posAtStart;return e.view.state.doc.slice(n,n+this.length)}destroy(){super.destroy(),this.widget.destroy(this.dom)}static of(e,n,r,i,s){return s||(s=e.toDOM(n),e.editable||(s.contentEditable="false")),new Gr(s,r,e,i)}}class wa extends Re{constructor(e){let n=document.createElement("img");n.className="cm-widgetBuffer",n.setAttribute("aria-hidden","true"),super(n,0,e)}get isHidden(){return!0}get overrideDOMText(){return te.empty}coordsIn(e,n,r){let i=this.dom.getBoundingClientRect();return r==null?i:va(i,n>0==r)}}class sT{constructor(e){this.index=0,this.beforeBreak=!1,this.parents=[],this.tile=e}advance(e,n,r){let{tile:i,index:s,beforeBreak:o,parents:l}=this;for(;e||n>0;)if(i.isComposite())if(o){if(!e)break;r&&r.break(),e--,o=!1}else if(s==i.children.length){if(!e&&!l.length)break;r&&r.leave(i),o=!!i.breakAfter,{tile:i,index:s}=l.pop(),s++}else{let a=i.children[s],c=a.breakAfter;(n>0?a.length<=e:a.length<e)&&(!r||r.skip(a,0,a.length)!==!1||!a.isComposite)?(o=!!c,s++,e-=a.length):(l.push({tile:i,index:s}),i=a,s=0,r&&a.isComposite()&&r.enter(a))}else{let a=i.length;if(s<a&&e){let c=Math.min(e,a-s);r&&r.skip(i,s,s+c),e-=c,s+=c}if(s==a)o=!!i.breakAfter,{tile:i,index:s}=l.pop(),s++;else if(!e)break}return this.tile=i,this.index=s,this.beforeBreak=o,this}get root(){return this.parents.length?this.parents[0].tile:this.tile}}class oT{constructor(e,n,r,i){this.from=e,this.to=n,this.wrapper=r,this.rank=i}}class lT{constructor(e,n,r){this.cache=e,this.root=n,this.blockWrappers=r,this.curLine=null,this.lastBlock=null,this.afterWidget=null,this.pos=0,this.wrappers=[],this.wrapperPos=0}addText(e,n,r,i){var s;this.flushBuffer();let o=this.ensureMarks(n,r),l=o.lastChild;if(l&&l.isText()&&!(l.flags&8)&&l.length+e.length<512){this.cache.reused.set(l,2);let a=o.children[o.children.length-1]=new Dr(l.dom,l.text+e);a.parent=o}else o.append(i||Dr.of(e,(s=this.cache.find(Dr))===null||s===void 0?void 0:s.dom));this.pos+=e.length,this.afterWidget=null}addComposition(e,n){let r=this.curLine;r.dom!=n.line.dom&&(r.setDOM(this.cache.reused.has(n.line)?Qc(n.line.dom):n.line.dom),this.cache.reused.set(n.line,2));let i=r;for(let l=n.marks.length-1;l>=0;l--){let a=n.marks[l],c=i.lastChild;if(c instanceof pt&&c.mark.eq(a.mark))c.dom!=a.dom&&c.setDOM(Qc(a.dom)),i=c;else{let{dom:u}=a;this.cache.reused.get(a)&&Re.get(a.dom)&&(u=Qc(a.dom));let h=pt.of(a.mark,u);i.append(h),i=h}this.cache.reused.set(a,2)}let s=Re.get(e.text);s&&this.cache.reused.set(s,2);let o=new Dr(e.text,e.text.nodeValue);o.flags|=8,this.pos=e.range.toB,i.append(o)}addInlineWidget(e,n,r){let i=this.afterWidget&&e.flags&48&&(this.afterWidget.flags&48)==(e.flags&48);i||this.flushBuffer();let s=this.ensureMarks(n,r);!i&&!(e.flags&16)&&s.append(this.getBuffer(1)),s.append(e),this.pos+=e.length,this.afterWidget=e}addMark(e,n,r){this.flushBuffer(),this.ensureMarks(n,r).append(e),this.pos+=e.length,this.afterWidget=null}addBlockWidget(e){this.getBlockPos().append(e),this.pos+=e.length,this.lastBlock=e,this.endLine()}continueWidget(e){let n=this.afterWidget||this.lastBlock;n.length+=e,this.pos+=e}addLineStart(e,n){var r;e||(e=rw);let i=Hi.start(e,n||((r=this.cache.find(Hi))===null||r===void 0?void 0:r.dom),!!n);this.getBlockPos().append(this.lastBlock=this.curLine=i)}addLine(e){this.getBlockPos().append(e),this.pos+=e.length,this.lastBlock=e,this.endLine()}addBreak(){this.lastBlock.flags|=1,this.endLine(),this.pos++}addLineStartIfNotCovered(e){this.blockPosCovered()||this.addLineStart(e)}ensureLine(e){this.curLine||this.addLineStart(e)}ensureMarks(e,n){var r;let i=this.curLine;for(let s=e.length-1;s>=0;s--){let o=e[s],l;if(n>0&&(l=i.lastChild)&&l instanceof pt&&l.mark.eq(o))i=l,n--;else{let a=pt.of(o,(r=this.cache.find(pt,c=>c.mark.eq(o)))===null||r===void 0?void 0:r.dom);i.append(a),i=a,n=0}}return i}endLine(){if(this.curLine){this.flushBuffer();let e=this.curLine.lastChild;(!e||!Mm(this.curLine,!1)||e.dom.nodeName!="BR"&&e.isWidget()&&!(_.ios&&Mm(this.curLine,!0)))&&this.curLine.append(this.cache.findWidget(Yc,0,32)||new Gr(Yc.toDOM(),0,Yc,32)),this.curLine=this.afterWidget=null}}updateBlockWrappers(){this.wrapperPos>this.pos+1e4&&(this.blockWrappers.goto(this.pos),this.wrappers.length=0);for(let e=this.wrappers.length-1;e>=0;e--)this.wrappers[e].to<this.pos&&this.wrappers.splice(e,1);for(let e=this.blockWrappers;e.value&&e.from<=this.pos;e.next())if(e.to>=this.pos){let n=e.rank*102+e.value.rank,r=new oT(e.from,e.to,e.value,n),i=this.wrappers.length;for(;i>0&&(this.wrappers[i-1].rank-r.rank||this.wrappers[i-1].to-r.to)<0;)i--;this.wrappers.splice(i,0,r)}this.wrapperPos=this.pos}getBlockPos(){var e;this.updateBlockWrappers();let n=this.root;for(let r of this.wrappers){let i=n.lastChild;if(r.from<this.pos&&i instanceof Pn&&i.wrapper.eq(r.wrapper))n=i;else{let s=Pn.of(r.wrapper,(e=this.cache.find(Pn,o=>o.wrapper.eq(r.wrapper)))===null||e===void 0?void 0:e.dom);n.append(s),n=s}}return n}blockPosCovered(){let e=this.lastBlock;return e!=null&&!e.breakAfter&&(!e.isWidget()||(e.flags&160)>0)}getBuffer(e){let n=2|(e<0?16:32),r=this.cache.find(wa,void 0,1);return r&&(r.flags=n),r||new wa(n)}flushBuffer(){this.afterWidget&&!(this.afterWidget.flags&32)&&(this.afterWidget.parent.append(this.getBuffer(-1)),this.afterWidget=null)}}class aT{constructor(e){this.skipCount=0,this.text="",this.textOff=0,this.cursor=e.iter()}skip(e){this.textOff+e<=this.text.length?this.textOff+=e:(this.skipCount+=e-(this.text.length-this.textOff),this.text="",this.textOff=0)}next(e){if(this.textOff==this.text.length){let{value:i,lineBreak:s,done:o}=this.cursor.next(this.skipCount);if(this.skipCount=0,o)throw new Error("Ran out of text content when drawing inline views");this.text=i;let l=this.textOff=Math.min(e,i.length);return s?null:i.slice(0,l)}let n=Math.min(this.text.length,this.textOff+e),r=this.text.slice(this.textOff,n);return this.textOff=n,r}}const Sa=[Gr,Hi,Dr,pt,wa,Pn,tc];for(let t=0;t<Sa.length;t++)Sa[t].bucket=t;class cT{constructor(e){this.view=e,this.buckets=Sa.map(()=>[]),this.index=Sa.map(()=>0),this.reused=new Map}add(e){let n=e.constructor.bucket,r=this.buckets[n];r.length<6?r.push(e):r[this.index[n]=(this.index[n]+1)%6]=e}find(e,n,r=2){let i=e.bucket,s=this.buckets[i],o=this.index[i];for(let l=0;l<s.length;l++){let a=(l+o)%s.length,c=s[a];if((!n||n(c))&&!this.reused.has(c))return s.splice(a,1),a<o&&this.index[i]--,this.reused.set(c,r),c}return null}findWidget(e,n,r){let i=this.buckets[0];if(i.length)for(let s=0,o=0;;s++){if(s==i.length){if(o)return null;o=1,s=0}let l=i[s];if(!this.reused.has(l)&&(o==0?l.widget.compare(e):l.widget.constructor==e.constructor&&e.updateDOM(l.dom,this.view,l.widget)))return i.splice(s,1),s<this.index[0]&&this.index[0]--,l.widget==e&&l.length==n&&(l.flags&497)==r?(this.reused.set(l,1),l):(this.reused.set(l,2),new Gr(l.dom,n,e,l.flags&-498|r))}}reuse(e){return this.reused.set(e,1),e}maybeReuse(e,n=2){if(!this.reused.has(e))return this.reused.set(e,n),e.dom}clear(){for(let e=0;e<this.buckets.length;e++)this.buckets[e].length=this.index[e]=0}}class uT{constructor(e,n,r,i,s){this.view=e,this.decorations=i,this.disallowBlockEffectsFor=s,this.openWidget=!1,this.openMarks=0,this.cache=new cT(e),this.text=new aT(e.state.doc),this.builder=new lT(this.cache,new tc(e,e.contentDOM),X.iter(r)),this.cache.reused.set(n,2),this.old=new sT(n),this.reuseWalker={skip:(o,l,a)=>{if(this.cache.add(o),o.isComposite())return!1},enter:o=>this.cache.add(o),leave:()=>{},break:()=>{}}}run(e,n){let r=n&&this.getCompositionContext(n.text);for(let i=0,s=0,o=0;;){let l=o<e.length?e[o++]:null,a=l?l.fromA:this.old.root.length;if(a>i){let c=a-i;this.preserve(c,!o,!l),i=a,s+=c}if(!l)break;n&&l.fromA<=n.range.fromA&&l.toA>=n.range.toA?(this.forward(l.fromA,n.range.fromA,n.range.fromA<n.range.toA?1:-1),this.emit(s,n.range.fromB),this.builder.flushBuffer(),this.cache.clear(),this.builder.addComposition(n,r),this.text.skip(n.range.toB-n.range.fromB),this.forward(n.range.fromA,l.toA),this.emit(n.range.toB,l.toB)):(this.forward(l.fromA,l.toA),this.emit(s,l.toB)),s=l.toB,i=l.toA}return this.builder.curLine&&this.builder.endLine(),this.builder.root}preserve(e,n,r){let i=dT(this.old),s=this.openMarks;this.old.advance(e,r?1:-1,{skip:(o,l,a)=>{if(o.isWidget())if(this.openWidget)this.builder.continueWidget(a-l);else{let c=a>0||l<o.length?Gr.of(o.widget,this.view,a-l,o.flags&496,this.cache.maybeReuse(o)):this.cache.reuse(o);c.flags&256?(c.flags&=-2,this.builder.addBlockWidget(c)):(this.builder.ensureLine(null),this.builder.addInlineWidget(c,i,s),s=i.length)}else if(o.isText())this.builder.ensureLine(null),!l&&a==o.length&&!this.cache.reused.has(o)?this.builder.addText(o.text,i,s,this.cache.reuse(o)):(this.cache.add(o),this.builder.addText(o.text.slice(l,a),i,s)),s=i.length;else if(o.isLine())o.flags&=-2,this.cache.reused.set(o,1),this.builder.addLine(o);else if(o instanceof wa)this.cache.add(o);else if(o instanceof pt)this.builder.ensureLine(null),this.builder.addMark(o,i,s),this.cache.reused.set(o,1),s=i.length;else return!1;this.openWidget=!1},enter:o=>{o.isLine()?this.builder.addLineStart(o.attrs,this.cache.maybeReuse(o)):(this.cache.add(o),o instanceof pt&&i.unshift(o.mark)),this.openWidget=!1},leave:o=>{o.isLine()?i.length&&(i.length=s=0):o instanceof pt&&(i.shift(),s=Math.min(s,i.length))},break:()=>{this.builder.addBreak(),this.openWidget=!1}}),this.text.skip(e)}emit(e,n){let r=null,i=this.builder,s=-1,o=X.spans(this.decorations,e,n,{point:(l,a,c,u,h,f)=>{if(c instanceof Ur){if(this.disallowBlockEffectsFor[f]){if(c.block)throw new RangeError("Block decorations may not be specified via plugins");if(a>this.view.state.doc.lineAt(l).to)throw new RangeError("Decorations that replace line breaks may not be specified via plugins")}if(s=u.length,h>u.length)i.continueWidget(a-l);else{let d=c.widget||(c.block?Ui.block:Ui.inline),p=hT(c),m=this.cache.findWidget(d,a-l,p)||Gr.of(d,this.view,a-l,p);c.block?(c.startSide>0&&i.addLineStartIfNotCovered(r),i.addBlockWidget(m)):(i.ensureLine(r),i.addInlineWidget(m,u,h))}r=null}else r=fT(r,c);a>l&&this.text.skip(a-l)},span:(l,a,c,u)=>{for(let h=l;h<a;){let f=this.text.next(Math.min(512,a-h));f==null?(i.addLineStartIfNotCovered(r),i.addBreak(),h++):(i.ensureLine(r),i.addText(f,c,h==l?u:c.length),h+=f.length),r=null}s=c.length}});s>-1&&(this.openWidget=o>s),this.openWidget||i.addLineStartIfNotCovered(r),this.openMarks=o}forward(e,n,r=1){n-e<=10?this.old.advance(n-e,r,this.reuseWalker):(this.old.advance(5,-1,this.reuseWalker),this.old.advance(n-e-10,-1),this.old.advance(5,r,this.reuseWalker))}getCompositionContext(e){let n=[],r=null;for(let i=e.parentNode;;i=i.parentNode){let s=Re.get(i);if(i==this.view.contentDOM)break;s instanceof pt?n.push(s):s!=null&&s.isLine()?r=s:s instanceof Pn||(i.nodeName=="DIV"&&!r?r=new Hi(i,rw):r||n.push(pt.of(new Mo({tagName:i.nodeName.toLowerCase(),attributes:Wb(i)}),i)))}return r?{line:r,marks:n}:null}}function Mm(t,e){let n=r=>{for(let i of r.children)if((e?i.isText():i.length)||n(i))return!0;return!1};return n(t)}function hT(t){let e=t.isReplace?(t.startSide<0?64:0)|(t.endSide>0?128:0):t.startSide>0?32:16;return t.block&&(e|=256),e}const rw={class:"cm-line"};function fT(t,e){let n=e.spec.attributes,r=e.spec.class;return!n&&!r||(t||(t={class:"cm-line"}),n&&ud(n,t),r&&(t.class+=" "+r)),t}function dT(t){let e=[];for(let n=t.parents.length;n>1;n--){let r=n==t.parents.length?t.tile:t.parents[n].tile;r instanceof pt&&e.push(r.mark)}return e}function Qc(t){let e=Re.get(t);return e&&e.setDOM(t.cloneNode()),t}class Ui extends qn{constructor(e){super(),this.tag=e}eq(e){return e.tag==this.tag}toDOM(){return document.createElement(this.tag)}updateDOM(e){return e.nodeName.toLowerCase()==this.tag}get isHidden(){return!0}}Ui.inline=new Ui("span");Ui.block=new Ui("div");const Yc=new class extends qn{toDOM(){return document.createElement("br")}get isHidden(){return!0}get editable(){return!0}};class Im{constructor(e){this.view=e,this.decorations=[],this.blockWrappers=[],this.dynamicDecorationMap=[!1],this.domChanged=null,this.hasComposition=null,this.editContextFormatting=U.none,this.lastCompositionAfterCursor=!1,this.minWidth=0,this.minWidthFrom=0,this.minWidthTo=0,this.impreciseAnchor=null,this.impreciseHead=null,this.forceSelection=!1,this.lastUpdate=Date.now(),this.updateDeco(),this.tile=new tc(e,e.contentDOM),this.updateInner([new Ot(0,0,0,e.state.doc.length)],null)}update(e){var n;let r=e.changedRanges;this.minWidth>0&&r.length&&(r.every(({fromA:u,toA:h})=>h<this.minWidthFrom||u>this.minWidthTo)?(this.minWidthFrom=e.changes.mapPos(this.minWidthFrom,1),this.minWidthTo=e.changes.mapPos(this.minWidthTo,1)):this.minWidth=this.minWidthFrom=this.minWidthTo=0),this.updateEditContextFormatting(e);let i=-1;this.view.inputState.composing>=0&&!this.view.observer.editContext&&(!((n=this.domChanged)===null||n===void 0)&&n.newSel?i=this.domChanged.newSel.head:!xT(e.changes,this.hasComposition)&&!e.selectionSet&&(i=e.state.selection.main.head));let s=i>-1?mT(this.view,e.changes,i):null;if(this.domChanged=null,this.hasComposition){let{from:u,to:h}=this.hasComposition;r=new Ot(u,h,e.changes.mapPos(u,-1),e.changes.mapPos(h,1)).addToSet(r.slice())}this.hasComposition=s?{from:s.range.fromB,to:s.range.toB}:null,(_.ie||_.chrome)&&!s&&e&&e.state.doc.lines!=e.startState.doc.lines&&(this.forceSelection=!0);let o=this.decorations,l=this.blockWrappers;this.updateDeco();let a=yT(o,this.decorations,e.changes);a.length&&(r=Ot.extendWithRanges(r,a));let c=ST(l,this.blockWrappers,e.changes);return c.length&&(r=Ot.extendWithRanges(r,c)),s&&!r.some(u=>u.fromA<=s.range.fromA&&u.toA>=s.range.toA)&&(r=s.range.addToSet(r.slice())),this.tile.flags&2&&r.length==0?!1:(this.updateInner(r,s),e.transactions.length&&(this.lastUpdate=Date.now()),!0)}updateInner(e,n){this.view.viewState.mustMeasureContent=!0;let{observer:r}=this.view;r.ignore(()=>{if(n||e.length){let o=this.tile,l=new uT(this.view,o,this.blockWrappers,this.decorations,this.dynamicDecorationMap);n&&Re.get(n.text)&&l.cache.reused.set(Re.get(n.text),2),this.tile=l.run(e,n),qh(o,l.cache.reused)}this.tile.dom.style.height=this.view.viewState.contentHeight/this.view.scaleY+"px",this.tile.dom.style.flexBasis=this.minWidth?this.minWidth+"px":"";let s=_.chrome||_.ios?{node:r.selectionRange.focusNode,written:!1}:void 0;this.tile.sync(s),s&&(s.written||r.selectionRange.focusNode!=s.node||!this.tile.dom.contains(s.node))&&(this.forceSelection=!0),this.tile.dom.style.height=""});let i=[];if(this.view.viewport.from||this.view.viewport.to<this.view.state.doc.length)for(let s of this.tile.children)s.isWidget()&&s.widget instanceof Xc&&i.push(s.dom);r.updateGaps(i)}updateEditContextFormatting(e){this.editContextFormatting=this.editContextFormatting.map(e.changes);for(let n of e.transactions)for(let r of n.effects)r.is(Jy)&&(this.editContextFormatting=r.value)}updateSelection(e=!1,n=!1){(e||!this.view.observer.selectionRange.focusNode)&&this.view.observer.readSelectionRange();let{dom:r}=this.tile,i=this.view.root.activeElement,s=i==r,o=!s&&!(this.view.state.facet(Rn)||r.tabIndex>-1)&&_s(r,this.view.observer.selectionRange)&&!(i&&r.contains(i));if(!(s||n||o))return;let l=this.forceSelection;this.forceSelection=!1;let a=this.view.state.selection.main,c,u;if(a.empty?u=c=this.inlineDOMNearPos(a.anchor,a.assoc||1):(u=this.inlineDOMNearPos(a.head,a.head==a.from?1:-1),c=this.inlineDOMNearPos(a.anchor,a.anchor==a.from?1:-1)),_.gecko&&a.empty&&!this.hasComposition&&pT(c)){let f=document.createTextNode("");this.view.observer.ignore(()=>c.node.insertBefore(f,c.node.childNodes[c.offset]||null)),c=u=new Gt(f,0),l=!0}let h=this.view.observer.selectionRange;(l||!h.focusNode||(!qs(c.node,c.offset,h.anchorNode,h.anchorOffset)||!qs(u.node,u.offset,h.focusNode,h.focusOffset))&&!this.suppressWidgetCursorChange(h,a))&&(this.view.observer.ignore(()=>{_.android&&_.chrome&&r.contains(h.focusNode)&&kT(h.focusNode,r)&&(r.blur(),r.focus({preventScroll:!0}));let f=uo(this.view.root);if(f)if(a.empty){if(_.gecko){let d=gT(c.node,c.offset);if(d&&d!=3){let p=(d==1?By:_y)(c.node,c.offset);p&&(c=new Gt(p.node,p.offset))}}f.collapse(c.node,c.offset),a.bidiLevel!=null&&f.caretBidiLevel!==void 0&&(f.caretBidiLevel=a.bidiLevel)}else if(f.extend){f.collapse(c.node,c.offset);try{f.extend(u.node,u.offset)}catch{}}else{let d=document.createRange();a.anchor>a.head&&([c,u]=[u,c]),d.setEnd(u.node,u.offset),d.setStart(c.node,c.offset),f.removeAllRanges(),f.addRange(d)}o&&this.view.root.activeElement==r&&(r.blur(),i&&i.focus())}),this.view.observer.setSelectionRange(c,u)),this.impreciseAnchor=c.precise?null:new Gt(h.anchorNode,h.anchorOffset),this.impreciseHead=u.precise?null:new Gt(h.focusNode,h.focusOffset)}suppressWidgetCursorChange(e,n){return this.hasComposition&&n.empty&&qs(e.focusNode,e.focusOffset,e.anchorNode,e.anchorOffset)&&this.posFromDOM(e.focusNode,e.focusOffset)==n.head}enforceCursorAssoc(){if(this.hasComposition)return;let{view:e}=this,n=e.state.selection.main,r=uo(e.root),{anchorNode:i,anchorOffset:s}=e.observer.selectionRange;if(!r||!n.empty||!n.assoc||!r.modify)return;let o=this.lineAt(n.head,n.assoc);if(!o)return;let l=o.posAtStart;if(n.head==l||n.head==l+o.length)return;let a=this.coordsAt(n.head,-1),c=this.coordsAt(n.head,1);if(!a||!c||a.bottom>c.top)return;let u=this.domAtPos(n.head+n.assoc,n.assoc);r.collapse(u.node,u.offset),r.modify("move",n.assoc<0?"forward":"backward","lineboundary"),e.observer.readSelectionRange();let h=e.observer.selectionRange;e.docView.posFromDOM(h.anchorNode,h.anchorOffset)!=n.from&&r.collapse(i,s)}posFromDOM(e,n){let r=this.tile.nearest(e);if(!r)return this.tile.dom.compareDocumentPosition(e)&2?0:this.view.state.doc.length;let i=r.posAtStart;if(r.isComposite()){let s;if(e==r.dom)s=r.dom.childNodes[n];else{let o=Nn(e)==0?0:n==0?-1:1;for(;;){let l=e.parentNode;if(l==r.dom)break;o==0&&l.firstChild!=l.lastChild&&(e==l.firstChild?o=-1:o=1),e=l}o<0?s=e:s=e.nextSibling}if(s==r.dom.firstChild)return i;for(;s&&!Re.get(s);)s=s.nextSibling;if(!s)return i+r.length;for(let o=0,l=i;;o++){let a=r.children[o];if(a.dom==s)return l;l+=a.length+a.breakAfter}}else return r.isText()?e==r.dom?i+n:i+(n?r.length:0):i}domAtPos(e,n){let{tile:r,offset:i}=this.tile.resolveBlock(e,n);return r.isWidget()?r.domPosFor(i,n):r.domIn(i,n)}inlineDOMNearPos(e,n){let r,i=-1,s=!1,o,l=-1,a=!1;return this.tile.blockTiles((c,u)=>{if(c.isWidget()){if(c.flags&32&&u>=e)return!0;c.flags&16&&(s=!0)}else{let h=u+c.length;if(u<=e&&(r=c,i=e-u,s=h<e),h>=e&&!o&&(o=c,l=e-u,a=u>e),u>e&&o)return!0}}),!r&&!o?this.domAtPos(e,n):(s&&o?r=null:a&&r&&(o=null),r&&n<0||!o?r.domIn(i,n):o.domIn(l,n))}coordsAt(e,n,r){let{tile:i,offset:s}=this.tile.resolveBlock(e,n);return i.isWidget()?i.widget instanceof Xc?null:i.coordsInWidget(s,n,!0):i.coordsIn(s,n,r)}lineAt(e,n){let{tile:r}=this.tile.resolveBlock(e,n);return r.isLine()?r:null}coordsForChar(e){let{tile:n,offset:r}=this.tile.resolveBlock(e,1);if(!n.isLine())return null;function i(s,o){if(s.isComposite())for(let l of s.children){if(l.length>=o){let a=i(l,o);if(a)return a}if(o-=l.length,o<0)break}else if(s.isText()&&o<s.length){let l=qe(s.text,o);if(l==o)return null;let a=ho(s.dom,o,l).getClientRects();for(let c=0;c<a.length;c++){let u=a[c];if(c==a.length-1||u.top<u.bottom&&u.left<u.right)return u}}return null}return i(n,r)}measureVisibleLineHeights(e){let n=[],{from:r,to:i}=e,s=this.view.contentDOM.clientWidth,o=s>Math.max(this.view.scrollDOM.clientWidth,this.minWidth)+1,l=-1,a=this.view.textDirection==ue.LTR,c=0,u=(h,f,d)=>{for(let p=0;p<h.children.length&&!(f>i);p++){let m=h.children[p],w=f+m.length,g=m.dom.getBoundingClientRect(),{height:v}=g;if(d&&!p&&(c+=g.top-d.top),m instanceof Pn)w>r&&u(m,f,g);else if(f>=r&&(c>0&&n.push(-c),n.push(v+c),c=0,o)){let y=m.dom.lastChild,x=y?Nl(y):[];if(x.length){let k=x[x.length-1],C=a?k.right-g.left:g.right-k.left;C>l&&(l=C,this.minWidth=s,this.minWidthFrom=f,this.minWidthTo=w)}}d&&p==h.children.length-1&&(c+=d.bottom-g.bottom),f=w+m.breakAfter}};return u(this.tile,0,null),n}textDirectionAt(e){let{tile:n}=this.tile.resolveBlock(e,1);return getComputedStyle(n.dom).direction=="rtl"?ue.RTL:ue.LTR}measureTextSize(){let e=this.tile.blockTiles(o=>{if(o.isLine()&&o.children.length&&o.length<=20){let l=0,a;for(let c of o.children){if(!c.isText()||/[^ -~]/.test(c.text))return;let u=Nl(c.dom);if(u.length!=1)return;l+=u[0].width,a=u[0].height}if(l)return{lineHeight:o.dom.getBoundingClientRect().height,charWidth:l/o.length,textHeight:a}}});if(e)return e;let n=document.createElement("div"),r,i,s;return n.className="cm-line",n.style.width="99999px",n.style.position="absolute",n.textContent="abc def ghi jkl mno pqr stu",this.view.observer.ignore(()=>{this.tile.dom.appendChild(n);let o=Nl(n.firstChild)[0];r=n.getBoundingClientRect().height,i=o&&o.width?o.width/27:7,s=o&&o.height?o.height:r,n.remove()}),{lineHeight:r,charWidth:i,textHeight:s}}computeBlockGapDeco(){let e=[],n=this.view.viewState;for(let r=0,i=0;;i++){let s=i==n.viewports.length?null:n.viewports[i],o=s?s.from-1:this.view.state.doc.length;if(o>r){let l=(n.lineBlockAt(o).bottom-n.lineBlockAt(r).top)/this.view.scaleY;e.push(U.replace({widget:new Xc(l),block:!0,inclusive:!0,isBlockGap:!0}).range(r,o))}if(!s)break;r=s.to+1}return U.set(e)}updateDeco(){let e=1,n=this.view.state.facet(Za).map(s=>(this.dynamicDecorationMap[e++]=typeof s=="function")?s(this.view):s),r=!1,i=this.view.state.facet(gd).map((s,o)=>{let l=typeof s=="function";return l&&(r=!0),l?s(this.view):s});for(i.length&&(this.dynamicDecorationMap[e++]=r,n.push(X.join(i))),this.decorations=[this.editContextFormatting,...n,this.computeBlockGapDeco(),this.view.viewState.lineGapDeco];e<this.decorations.length;)this.dynamicDecorationMap[e++]=!1;this.blockWrappers=this.view.state.facet(ew).map(s=>typeof s=="function"?s(this.view):s)}scrollIntoView(e){if(e.isSnapshot){let c=this.view.viewState.lineBlockAt(e.range.head);this.view.scrollDOM.scrollTop=c.top-e.yMargin,this.view.scrollDOM.scrollLeft=e.xMargin;return}for(let c of this.view.state.facet(Xy))try{if(c(this.view,e.range,e))return!0}catch(u){st(this.view.state,u,"scroll handler")}let{range:n}=e,r=this.coordsAt(n.head,n.assoc||(n.head>n.anchor?-1:1)),i;if(!r)return;!n.empty&&(i=this.coordsAt(n.anchor,n.anchor>n.head?-1:1))&&(r={left:Math.min(r.left,i.left),top:Math.min(r.top,i.top),right:Math.max(r.right,i.right),bottom:Math.max(r.bottom,i.bottom)});let s=vd(this.view),o={left:r.left-s.left,top:r.top-s.top,right:r.right+s.right,bottom:r.bottom+s.bottom},{offsetWidth:l,offsetHeight:a}=this.view.scrollDOM;if(jb(this.view.scrollDOM,o,n.head<n.anchor?-1:1,e.x,e.y,Math.max(Math.min(e.xMargin,l),-l),Math.max(Math.min(e.yMargin,a),-a),this.view.textDirection==ue.LTR),window.visualViewport&&window.innerHeight-window.visualViewport.height>1&&(r.top>window.visualViewport.offsetTop+window.visualViewport.height||r.bottom<window.visualViewport.offsetTop)){let c=this.view.docView.lineAt(n.head,1);if(c){let u=Ly(c.dom);c.dom.scrollIntoView({block:"nearest"}),Dy(u,!1)}}}lineHasWidget(e){let n=r=>r.isWidget()||r.children.some(n);return n(this.tile.resolveBlock(e,1).tile)}destroy(){qh(this.tile)}}function qh(t,e){let n=e==null?void 0:e.get(t);if(n!=1){n==null&&t.destroy();for(let r of t.children)qh(r,e)}}function pT(t){return Fe(t.node)&&t.node.firstChild&&(t.offset==0||t.node.childNodes[t.offset-1].contentEditable=="false")&&(t.offset==t.node.childNodes.length||t.node.childNodes[t.offset].contentEditable=="false")}function iw(t,e){let n=t.observer.selectionRange;if(!n.focusNode)return null;let r=By(n.focusNode,n.focusOffset),i=_y(n.focusNode,n.focusOffset),s=r||i;if(i&&r&&i.node!=r.node){let l=Re.get(i.node);if(!l||l.isText()&&l.text!=i.node.nodeValue)s=i;else if(t.docView.lastCompositionAfterCursor){let a=Re.get(r.node);!a||a.isText()&&a.text!=r.node.nodeValue||(s=i)}}if(t.docView.lastCompositionAfterCursor=s!=r,!s)return null;let o=e-s.offset;return{from:o,to:o+s.node.nodeValue.length,node:s.node}}function mT(t,e,n){let r=iw(t,n);if(!r)return null;let{node:i,from:s,to:o}=r,l=i.nodeValue;if(/[\n\r]/.test(l)||t.state.doc.sliceString(r.from,r.to)!=l)return null;let a=e.invertedDesc;return{range:new Ot(a.mapPos(s),a.mapPos(o),s,o),text:i}}function gT(t,e){return Fe(t)?(e&&t.childNodes[e-1].contentEditable=="false"?1:0)|(e<t.childNodes.length&&t.childNodes[e].contentEditable=="false"?2:0):0}let vT=class{constructor(){this.changes=[]}compareRange(e,n){Ai(e,n,this.changes)}comparePoint(e,n){Ai(e,n,this.changes)}boundChange(e){Ai(e,e,this.changes)}};function yT(t,e,n){let r=new vT;return X.compare(t,e,n,r),r.changes}class wT{constructor(){this.changes=[]}compareRange(e,n){Ai(e,n,this.changes)}comparePoint(){}boundChange(e){Ai(e,e,this.changes)}}function ST(t,e,n){let r=new wT;return X.compare(t,e,n,r),r.changes}function kT(t,e){for(let n=t;n&&n!=e;n=n.assignedSlot||n.parentNode)if(Fe(n)&&n.contentEditable=="false")return!0;return!1}function xT(t,e){let n=!1;return e&&t.iterChangedRanges((r,i)=>{r<e.to&&i>e.from&&(n=!0)}),n}class Xc extends qn{constructor(e){super(),this.height=e}toDOM(){let e=document.createElement("div");return e.className="cm-gap",this.updateDOM(e),e}eq(e){return e.height==this.height}updateDOM(e){return e.style.height=this.height+"px",!0}get editable(){return!0}get estimatedHeight(){return this.height}ignoreEvent(){return!1}}function CT(t,e,n=1){let r=t.charCategorizer(e),i=t.doc.lineAt(e),s=e-i.from;if(i.length==0)return R.cursor(e);s==0?n=1:s==i.length&&(n=-1);let o=s,l=s;n<0?o=qe(i.text,s,!1):l=qe(i.text,s);let a=r(i.text.slice(o,l));for(;o>0;){let c=qe(i.text,o,!1);if(r(i.text.slice(c,o))!=a)break;o=c}for(;l<i.length;){let c=qe(i.text,l);if(r(i.text.slice(l,c))!=a)break;l=c}return R.undirectionalRange(o+i.from,l+i.from)}function bT(t,e,n,r,i){let s=Math.round((r-e.left)*t.defaultCharacterWidth);if(t.lineWrapping&&n.height>t.defaultLineHeight*1.5){let l=t.viewState.heightOracle.textHeight,a=Math.floor((i-n.top-(t.defaultLineHeight-l)*.5)/l);s+=a*t.viewState.heightOracle.lineLength}let o=t.state.sliceDoc(n.from,n.to);return n.from+Rh(o,s,t.state.tabSize)}function Wh(t,e,n){let r=t.lineBlockAt(e);if(Array.isArray(r.type)){let i;for(let s of r.type){if(s.from>e)break;if(!(s.to<e)){if(s.from<e&&s.to>e)return s;(!i||s.type==Ge.Text&&(i.type!=s.type||(n<0?s.from<e:s.to>e)))&&(i=s)}}return i||r}return r}function TT(t,e,n,r){let i=Wh(t,e.head,e.assoc||-1),s=!r||i.type!=Ge.Text||!(t.lineWrapping||i.widgetLineBreaks)?null:t.coordsAtPos(e.assoc<0&&e.head>i.from?e.head-1:e.head);if(s){let o=t.dom.getBoundingClientRect(),l=t.textDirectionAt(i.from),a=t.posAtCoords({x:n==(l==ue.LTR)?o.right-1:o.left+1,y:(s.top+s.bottom)/2});if(a!=null)return R.cursor(a,n?-1:1)}return R.cursor(n?i.to:i.from,n?-1:1)}function Am(t,e,n,r){let i=t.state.doc.lineAt(e.head),s=t.bidiSpans(i),o=t.textDirectionAt(i.from);for(let l=e,a=null;;){let c=Zb(i,s,o,l,n),u=jy;if(!c){if(i.number==(n?t.state.doc.lines:1))return l;u=`
`,i=t.state.doc.line(i.number+(n?1:-1)),s=t.bidiSpans(i),c=n?R.cursor(i.from,-1):R.cursor(i.to,1)}if(a){if(!a(u))return l}else{if(!r)return c;a=r(u)}l=c}}function RT(t,e,n){let r=t.state.charCategorizer(e),i=r(n);return s=>{let o=r(s);return i==we.Space&&(i=o),i==o}}function MT(t,e,n,r){let i=e.head,s=n?1:-1;if(i==(n?t.state.doc.length:0))return R.cursor(i,e.assoc);let o=e.goalColumn,l,a=t.contentDOM.getBoundingClientRect(),c=t.coordsAtPos(i,e.assoc||((e.empty?n:e.head==e.from)?1:-1)),u=t.documentTop;if(c)o==null&&(o=c.left-a.left),l=s<0?c.top:c.bottom;else{let p=t.viewState.lineBlockAt(i);o==null&&(o=Math.min(a.right-a.left,t.defaultCharacterWidth*(i-p.from))),l=(s<0?p.top:p.bottom)+u}let h=a.left+o,f=t.viewState.heightOracle.textHeight>>1,d=r??f;for(let p=0;;p+=f){let m=l+(d+p)*s,w=Fh(t,{x:h,y:m},!1,s);if(n?m>a.bottom:m<a.top)return R.cursor(w.pos,w.assoc);let g=t.coordsAtPos(w.pos,w.assoc),v=g?(g.top+g.bottom)/2:0;if(!g||(n?v>l:v<l))return R.cursor(w.pos,w.assoc,void 0,o)}}function Ws(t,e,n){for(;;){let r=0;for(let i of t)i.between(e-1,e+1,(s,o,l)=>{if(e>s&&e<o){let a=r||n||(e-s<o-e?-1:1);e=a<0?s:o,r=a}});if(!r)return e}}function sw(t,e){let n=null;for(let r=0;r<e.ranges.length;r++){let i=e.ranges[r],s=null;if(i.empty){let o=Ws(t,i.from,0);o!=i.from&&(s=R.cursor(o,-1))}else{let o=Ws(t,i.from,-1),l=Ws(t,i.to,1);(o!=i.from||l!=i.to)&&(i.undirectional?s=R.undirectionalRange(i.from,i.to):s=R.range(i.from==i.anchor?o:l,i.from==i.head?o:l))}s&&(n||(n=e.ranges.slice()),n[r]=s)}return n?R.create(n,e.mainIndex):e}function Jc(t,e,n){let r=Ws(t.state.facet(Ao).map(i=>i(t)),n.from,e.head>n.from?-1:1);return r==n.from?n:R.cursor(r,r<n.from?1:-1)}class dn{constructor(e,n){this.pos=e,this.assoc=n}}function Fh(t,e,n,r){let i=t.contentDOM.getBoundingClientRect(),s=i.top+t.viewState.paddingTop,{x:o,y:l}=e,a=l-s,c;for(;;){if(a<0)return new dn(0,1);if(a>t.viewState.docHeight)return new dn(t.state.doc.length,-1);if(c=t.elementAtHeight(a),r==null)break;if(c.type==Ge.Text){if(r<0?c.to<t.viewport.from:c.from>t.viewport.to)break;let f=t.docView.coordsAt(r<0?c.from:c.to,r>0?-1:1);if(f&&(r<0?f.top<=a+s:f.bottom>=a+s))break}let h=t.viewState.heightOracle.textHeight/2;a=r>0?c.bottom+h:c.top-h}if(t.viewport.from>=c.to||t.viewport.to<=c.from){if(n)return null;if(c.type==Ge.Text){let h=bT(t,i,c,o,l);return new dn(h,h==c.from?1:-1)}}if(c.type!=Ge.Text)return a<(c.top+c.bottom)/2?new dn(c.from,1):new dn(c.to,-1);let u=t.docView.lineAt(c.from,2);return(!u||u.length!=c.length)&&(u=t.docView.lineAt(c.from,-2)),new IT(t,o,l,t.textDirectionAt(c.from)).scanTile(u,c.from)}class IT{constructor(e,n,r,i){this.view=e,this.x=n,this.y=r,this.baseDir=i,this.line=null,this.spans=null}bidiSpansAt(e){return(!this.line||this.line.from>e||this.line.to<e)&&(this.line=this.view.state.doc.lineAt(e),this.spans=this.view.bidiSpans(this.line)),this}baseDirAt(e,n){let{line:r,spans:i}=this.bidiSpansAt(e);return i[pn.find(i,e-r.from,-1,n)].level==this.baseDir}dirAt(e,n){let{line:r,spans:i}=this.bidiSpansAt(e);return i[pn.find(i,e-r.from,-1,n)].dir}bidiIn(e,n){let{spans:r,line:i}=this.bidiSpansAt(e);return r.length>1||r.length&&(r[0].level!=this.baseDir||r[0].to+i.from<n)}scan(e,n,r=!1){let i=0,s=e.length-1,o=new Set,l=this.bidiIn(e[0],e[s]),a,c,u=-1,h=1e9,f;e:for(;i<s;){let p=s-i,m=i+s>>1;t:if(o.has(m)){for(let v=1;v<p;v++){let y=m+v;if(y>=s&&(y-=p),!o.has(y)){m=y;break t}}break e}o.add(m);let w=n(m),g=0;if(w)for(let v=0;v<w.length;v++){let y=w[v];if(!(y.width==0&&w.length>1))if(y.bottom<this.y)(!a||a.bottom<y.bottom)&&(a=y),g=1;else if(y.top>this.y)(!c||c.top>y.top)&&(c=y),g=-1;else{let x=y.left>this.x?this.x-y.left:y.right<this.x?this.x-y.right:0,k=Math.abs(x);k<h&&(u=m,h=k,f=y),x&&(g=x<0==(this.baseDir==ue.LTR)?-1:1)}}g==-1&&(!l||this.baseDirAt(e[m],1))?s=m:g==1&&(!l||this.baseDirAt(e[m+1],-1))&&(i=m+1)}if(!f){if(!c&&!a)return{i:0,after:!1};let p=a&&(!c||this.y-a.bottom<c.top-this.y)?a:c;return this.y=(p.top+p.bottom)/2,this.scan(e,n,!0)}if(h&&!r){let{top:p,bottom:m}=f;if(a&&a.bottom>(p+p+m)/3)return this.y=a.bottom-1,this.scan(e,n,!0);if(c&&c.top<(p+m+m)/3)return this.y=c.top+1,this.scan(e,n,!0)}let d=(l?this.dirAt(e[u],1):this.baseDir)==ue.LTR;return{i:u,after:this.x>(f.left+f.right)/2==d}}scanText(e,n){let r=[];for(let s=0;s<e.length;s=qe(e.text,s))r.push(n+s);r.push(n+e.length);let i=this.scan(r,s=>{let o=r[s]-n,l=r[s+1]-n;return ho(e.dom,o,l).getClientRects()});return i.after?new dn(r[i.i+1],-1):new dn(r[i.i],1)}scanTile(e,n){if(!e.length)return new dn(n,1);if(e.children.length==1){let l=e.children[0];if(l.isText())return this.scanText(l,n);if(l.isComposite())return this.scanTile(l,n)}let r=[n];for(let l=0,a=n;l<e.children.length;l++)r.push(a+=e.children[l].length);let i=this.scan(r,l=>{let a=e.children[l];return a.flags&48?null:(Fe(a.dom)?a.dom:ho(a.dom,0,a.length)).getClientRects()}),s=e.children[i.i],o=r[i.i];return s.isText()?this.scanText(s,o):s.isComposite()?this.scanTile(s,o):i.after?new dn(r[i.i+1],-1):new dn(o,1)}}const ri="￿";class AT{constructor(e,n){this.points=e,this.view=n,this.text="",this.lineSeparator=n.state.facet(ee.lineSeparator)}append(e){this.text+=e}lineBreak(){this.text+=ri}readRange(e,n){if(!e)return this;let r=e.parentNode;for(let i=e;;){this.findPointBefore(r,i);let s=this.text.length;this.readNode(i);let o=Re.get(i),l=i.nextSibling;if(l==n){o!=null&&o.breakAfter&&!l&&r!=this.view.contentDOM&&this.lineBreak();break}let a=Re.get(l);(o&&a?o.breakAfter:(o?o.breakAfter:ga(i))||ga(l)&&(i.nodeName!="BR"||o!=null&&o.isWidget())&&this.text.length>s)&&!ET(l,n)&&this.lineBreak(),i=l}return this.findPointBefore(r,n),this}readTextNode(e){let n=e.nodeValue;for(let r of this.points)r.node==e&&(r.pos=this.text.length+Math.min(r.offset,n.length));for(let r=0,i=this.lineSeparator?null:/\r\n?|\n/g;;){let s=-1,o=1,l;if(this.lineSeparator?(s=n.indexOf(this.lineSeparator,r),o=this.lineSeparator.length):(l=i.exec(n))&&(s=l.index,o=l[0].length),this.append(n.slice(r,s<0?n.length:s)),s<0)break;if(this.lineBreak(),o>1)for(let a of this.points)a.node==e&&a.pos>this.text.length&&(a.pos-=o-1);r=s+o}}readNode(e){let n=Re.get(e),r=n&&n.overrideDOMText;if(r!=null){this.findPointInside(e,r.length);for(let i=r.iter();!i.next().done;)i.lineBreak?this.lineBreak():this.append(i.value)}else Vi(e)?this.readTextNode(e):e.nodeName=="BR"?e.nextSibling&&this.lineBreak():Fe(e)&&this.readRange(e.firstChild,null)}findPointBefore(e,n){for(let r of this.points)r.node==e&&e.childNodes[r.offset]==n&&(r.pos=this.text.length)}findPointInside(e,n){for(let r of this.points)(Vi(e)?r.node==e:e.contains(r.node))&&(r.pos=this.text.length+(PT(e,r.node,r.offset)?n:0))}}function PT(t,e,n){for(;;){if(!e||n<Nn(e))return!1;if(e==t)return!0;n=dr(e)+1,e=e.parentNode}}function ET(t,e){let n;for(;!(t==e||!t);t=t.nextSibling){let r=Re.get(t);if(!(r!=null&&r.isWidget()))return!1;r&&(n||(n=[])).push(r)}if(n)for(let r of n){let i=r.overrideDOMText;if(i!=null&&i.length)return!1}return!0}class Pm{constructor(e,n){this.node=e,this.offset=n,this.pos=-1}}class LT{constructor(e,n,r,i){this.typeOver=i,this.bounds=null,this.text="",this.domChanged=n>-1;let{impreciseHead:s,impreciseAnchor:o}=e.docView,l=e.state.selection;if(e.state.readOnly&&n>-1)this.newSel=null;else if(n>-1&&(this.bounds=ow(e.docView.tile,n,r,0))){let a=s||o?[]:OT(e),c=new AT(a,e);c.readRange(this.bounds.startDOM,this.bounds.endDOM),this.text=c.text,this.newSel=NT(a,this.bounds.from)}else{let a=e.observer.selectionRange,c=s&&s.node==a.focusNode&&s.offset==a.focusOffset||!Dh(e.contentDOM,a.focusNode)?l.main.head:e.docView.posFromDOM(a.focusNode,a.focusOffset),u=o&&o.node==a.anchorNode&&o.offset==a.anchorOffset||!Dh(e.contentDOM,a.anchorNode)?l.main.anchor:e.docView.posFromDOM(a.anchorNode,a.anchorOffset),h=e.viewport;if((_.ios||_.chrome)&&c!=u&&Math.min(c,u)<=l.main.from&&Math.max(c,u)>=l.main.to&&(h.from>0||h.to<e.state.doc.length)){let f=Math.min(c,u),d=Math.max(c,u),p=h.from-f,m=h.to-d;(p==0||p==1||f==0)&&(m==0||m==-1||d==e.state.doc.length)&&(c=0,u=e.state.doc.length)}if(e.inputState.composing>-1&&l.ranges.length>1)this.newSel=l.replaceRange(R.range(u,c));else if(e.lineWrapping&&u==c&&!(l.main.empty&&l.main.head==c)&&e.inputState.lastTouchTime>Date.now()-100){let f=e.coordsAtPos(c,-1),d=0;f&&(d=e.inputState.lastTouchY<=f.bottom?-1:1),this.newSel=R.create([R.cursor(c,d)])}else this.newSel=R.single(u,c)}}}function ow(t,e,n,r){if(t.isComposite()){let i=-1,s=-1,o=-1,l=-1;for(let a=0,c=r,u=r;a<t.children.length;a++){let h=t.children[a],f=c+h.length;if(c<e&&f>n)return ow(h,e,n,c);if(f>=e&&i==-1&&(i=a,s=c),c>n&&h.dom.parentNode==t.dom){o=a,l=u;break}u=f,c=f+h.breakAfter}return{from:s,to:l<0?r+t.length:l,startDOM:(i?t.children[i-1].dom.nextSibling:null)||t.dom.firstChild,endDOM:o<t.children.length&&o>=0?t.children[o].dom:null}}else return t.isText()?{from:r,to:r+t.length,startDOM:t.dom,endDOM:t.dom.nextSibling}:null}function lw(t,e){let n,{newSel:r}=e,{state:i}=t,s=i.selection.main,o=t.inputState.lastKeyTime>Date.now()-100?t.inputState.lastKeyCode:-1;if(e.bounds){let{from:l,to:a}=e.bounds,c=s.from,u=null;(o===8||_.android&&e.text.length<a-l)&&(c=s.to,u="end");let h=i.doc.sliceString(l,a,ri),f,d;!s.empty&&s.from>=l&&s.to<=a&&(e.typeOver||h!=e.text)&&h.slice(0,s.from-l)==e.text.slice(0,s.from-l)&&h.slice(s.to-l)==e.text.slice(f=e.text.length-(h.length-(s.to-l)))?n={from:s.from,to:s.to,insert:te.of(e.text.slice(s.from-l,f).split(ri))}:(d=aw(h,e.text,c-l,u))&&(_.chrome&&o==13&&d.toB==d.from+2&&e.text.slice(d.from,d.toB)==ri+ri&&d.toB--,n={from:l+d.from,to:l+d.toA,insert:te.of(e.text.slice(d.from,d.toB).split(ri))})}else r&&(!t.hasFocus&&i.facet(Rn)||zh(r,s))&&(r=null);if(!n&&!r)return!1;if((_.mac||_.android)&&n&&n.from==n.to&&n.from==s.head-1&&/^\. ?$/.test(n.insert.toString())&&t.contentDOM.getAttribute("autocorrect")=="off"?(r&&n.insert.length==2&&(r=R.single(r.main.anchor-1,r.main.head-1)),n={from:n.from,to:n.to,insert:te.of([n.insert.toString().replace("."," ")])}):i.doc.lineAt(s.from).to<s.to&&t.docView.lineHasWidget(s.to)&&t.inputState.insertingTextAt>Date.now()-50?n={from:s.from,to:s.to,insert:i.toText(t.inputState.insertingText)}:_.chrome&&n&&n.from==n.to&&n.from==s.head&&n.insert.toString()==`
 `&&t.lineWrapping&&(r&&(r=R.single(r.main.anchor-1,r.main.head-1)),n={from:s.from,to:s.to,insert:te.of([" "])}),n)return yd(t,n,r,o);if(r&&!zh(r,s)){let l=!1,a="select";return t.inputState.lastSelectionTime>Date.now()-50&&(t.inputState.lastSelectionOrigin=="select"&&(l=!0),a=t.inputState.lastSelectionOrigin,a=="select.pointer"&&(r=sw(i.facet(Ao).map(c=>c(t)),r))),t.dispatch({selection:r,scrollIntoView:l,userEvent:a}),!0}else return!1}function yd(t,e,n,r=-1){if(_.ios&&t.inputState.flushIOSKey(e))return!0;let i=t.state.selection.main;if(_.android&&(e.to==i.to&&(e.from==i.from||e.from==i.from-1&&t.state.sliceDoc(e.from,i.from)==" ")&&e.insert.length==1&&e.insert.lines==2&&Pi(t.contentDOM,"Enter",13)||(e.from==i.from-1&&e.to==i.to&&e.insert.length==0||r==8&&e.insert.length<e.to-e.from&&e.to>i.head)&&Pi(t.contentDOM,"Backspace",8)||e.from==i.from&&e.to==i.to+1&&e.insert.length==0&&Pi(t.contentDOM,"Delete",46)))return!0;let s=e.insert.toString();t.inputState.composing>=0&&t.inputState.composing++;let o,l=()=>o||(o=DT(t,e,n));return t.state.facet(Gy).some(a=>a(t,e.from,e.to,s,l))||t.dispatch(l()),!0}function DT(t,e,n){let r,i=t.state,s=i.selection.main,o=-1;if(e.from==e.to&&e.from<s.from||e.from>s.to){let a=e.from<s.from?-1:1,c=a<0?s.from:s.to,u=Ws(i.facet(Ao).map(h=>h(t)),c,a);e.from==u&&(o=u)}if(o>-1)r={changes:e,selection:R.cursor(e.from+e.insert.length,-1)};else if(e.from>=s.from&&e.to<=s.to&&e.to-e.from>=(s.to-s.from)/3&&(!n||n.main.empty&&n.main.from==e.from+e.insert.length)&&t.inputState.composing<0){let a=s.from<e.from?i.sliceDoc(s.from,e.from):"",c=s.to>e.to?i.sliceDoc(e.to,s.to):"";r=i.replaceSelection(t.state.toText(a+e.insert.sliceString(0,void 0,t.state.lineBreak)+c))}else{let a=i.changes(e),c=n&&n.main.to<=a.newLength?n.main:void 0;if(i.selection.ranges.length>1&&(t.inputState.composing>=0||t.inputState.compositionPendingChange)&&e.to<=s.to+10&&e.to>=s.to-10){let u=t.state.sliceDoc(e.from,e.to),h,f=n&&iw(t,n.main.head);if(f){let p=e.insert.length-(e.to-e.from);h={from:f.from,to:f.to-p}}else h=t.state.doc.lineAt(s.head);let d=s.to-e.to;r=i.changeByRange(p=>{if(p.from==s.from&&p.to==s.to)return{changes:a,range:c||p.map(a)};let m=p.to-d,w=m-u.length;if(t.state.sliceDoc(w,m)!=u||m>=h.from&&w<=h.to)return{range:p};let g=i.changes({from:w,to:m,insert:e.insert}),v=p.to-s.to;return{changes:g,range:c?R.range(Math.max(0,c.anchor+v),Math.max(0,c.head+v)):p.map(g)}})}else r={changes:a,selection:c&&i.selection.replaceRange(c)}}let l="input.type";return(t.composing||t.inputState.compositionPendingChange&&t.inputState.compositionEndedAt>Date.now()-50)&&(t.inputState.compositionPendingChange=!1,l+=".compose",t.inputState.compositionFirstChange&&(l+=".start",t.inputState.compositionFirstChange=!1)),i.update(r,{userEvent:l,scrollIntoView:!0})}function aw(t,e,n,r){let i=Math.min(t.length,e.length),s=0;for(;s<i&&t.charCodeAt(s)==e.charCodeAt(s);)s++;if(s==i&&t.length==e.length)return null;let o=t.length,l=e.length;for(;o>0&&l>0&&t.charCodeAt(o-1)==e.charCodeAt(l-1);)o--,l--;if(r=="end"){let a=Math.max(0,s-Math.min(o,l));n-=o+a-s}if(o<s&&t.length<e.length){let a=n<=s&&n>=o?s-n:0;s-=a,l=s+(l-o),o=s}else if(l<s){let a=n<=s&&n>=l?s-n:0;s-=a,o=s+(o-l),l=s}return{from:s,toA:o,toB:l}}function OT(t){let e=[];if(t.root.activeElement!=t.contentDOM)return e;let{anchorNode:n,anchorOffset:r,focusNode:i,focusOffset:s}=t.observer.selectionRange;return n&&(e.push(new Pm(n,r)),(i!=n||s!=r)&&e.push(new Pm(i,s))),e}function NT(t,e){if(t.length==0)return null;let n=t[0].pos,r=t.length==2?t[1].pos:n;return n<0||r<0?null:n==r?R.create([R.cursor(r+e,-1)]):R.single(n+e,r+e)}function zh(t,e){return e.head==t.main.head&&e.anchor==t.main.anchor}class BT{setSelectionOrigin(e){this.lastSelectionOrigin=e,this.lastSelectionTime=Date.now()}constructor(e){this.view=e,this.lastKeyCode=0,this.lastKeyTime=0,this.touchActive=!1,this.lastTouchTime=0,this.lastTouchX=0,this.lastTouchY=0,this.lastFocusTime=0,this.lastScrollTop=0,this.lastScrollLeft=0,this.lastWheelEvent=0,this.pendingIOSKey=void 0,this.lastIOSMomentumScroll=0,this.tabFocusMode=-1,this.lastSelectionOrigin=null,this.lastSelectionTime=0,this.lastContextMenu=0,this.scrollHandlers=[],this.handlers=Object.create(null),this.composing=-1,this.compositionFirstChange=null,this.compositionEndedAt=0,this.compositionPendingKey=!1,this.compositionPendingChange=!1,this.insertingText="",this.insertingTextAt=0,this.mouseSelection=null,this.draggedContent=null,this.handleEvent=this.handleEvent.bind(this),this.notifiedFocused=e.hasFocus,_.safari&&e.contentDOM.addEventListener("input",()=>null),_.gecko&&JT(e.contentDOM.ownerDocument)}handleEvent(e){!UT(this.view,e)||this.ignoreDuringComposition(e)||e.type=="keydown"&&this.keydown(e)||(this.view.updateState!=0?Promise.resolve().then(()=>this.runHandlers(e.type,e)):this.runHandlers(e.type,e))}runHandlers(e,n){let r=this.handlers[e];if(r){for(let i of r.observers)i(this.view,n);for(let i of r.handlers){if(n.defaultPrevented)break;if(i(this.view,n)){n.preventDefault();break}}}}ensureHandlers(e){let n=qT(e),r=this.handlers,i=this.view.contentDOM;for(let s in n)if(s!="scroll"){let o=!n[s].handlers.length,l=r[s];l&&o!=!l.handlers.length&&(i.removeEventListener(s,this.handleEvent),l=null),l||i.addEventListener(s,this.handleEvent,{passive:o})}for(let s in r)s!="scroll"&&!n[s]&&i.removeEventListener(s,this.handleEvent);this.handlers=n}keydown(e){if(this.lastKeyCode=e.keyCode,this.lastKeyTime=Date.now(),e.keyCode==9&&this.tabFocusMode>-1&&(!this.tabFocusMode||Date.now()<=this.tabFocusMode))return!0;if(this.tabFocusMode>0&&e.keyCode!=27&&uw.indexOf(e.keyCode)<0&&(this.tabFocusMode=-1),_.android&&_.chrome&&!e.synthetic&&(e.keyCode==13||e.keyCode==8))return this.view.observer.delayAndroidKey(e.key,e.keyCode),!0;if(_.ios&&!e.synthetic&&!e.altKey&&!e.metaKey&&(cw.some(n=>n.keyCode==e.keyCode)&&!e.ctrlKey||WT.indexOf(e.key)>-1&&e.ctrlKey)){let n={ctrlKey:e.ctrlKey,altKey:e.altKey,metaKey:e.metaKey,shiftKey:e.shiftKey};n.shiftKey&&_.ios&&!/^(off|none)$/.test(this.view.contentDOM.autocapitalize)&&_T(this.view.win)&&(n.shiftKey=!1);let r=this.pendingIOSKey={key:e.key,keyCode:e.keyCode,mods:n};return setTimeout(()=>{this.pendingIOSKey==r&&this.flushIOSKey()},50),!0}return e.keyCode!=229&&this.view.observer.forceFlush(),!1}flushIOSKey(e){let n=this.pendingIOSKey;return!n||this.view.observer.pendingRecords().length||n.key=="Enter"&&e&&e.from<e.to&&/^\S+$/.test(e.insert.toString())?!1:(this.pendingIOSKey=void 0,Pi(this.view.contentDOM,n.key,n.keyCode,n.mods))}ignoreDuringComposition(e){return!/^key/.test(e.type)||e.synthetic?!1:this.composing>0?!0:_.safari&&!_.ios&&this.compositionPendingKey&&Date.now()-this.compositionEndedAt<100?(this.compositionPendingKey=!1,!0):!1}startMouseSelection(e){this.mouseSelection&&this.mouseSelection.destroy(),this.mouseSelection=e}update(e){this.view.observer.update(e),this.mouseSelection&&this.mouseSelection.update(e),this.draggedContent&&e.docChanged&&(this.draggedContent=this.draggedContent.map(e.changes)),e.transactions.length&&(this.lastKeyCode=this.lastSelectionTime=0)}destroy(){this.mouseSelection&&this.mouseSelection.destroy()}}function _T(t){return t.visualViewport?t.visualViewport.height*t.visualViewport.scale/t.document.documentElement.clientHeight<.85:!1}function Em(t,e){return(n,r)=>{try{return e.call(t,r,n)}catch(i){st(n.state,i)}}}function qT(t){let e=Object.create(null);function n(r){return e[r]||(e[r]={observers:[],handlers:[]})}for(let r of t){let i=r.spec,s=i&&i.plugin.domEventHandlers,o=i&&i.plugin.domEventObservers;if(s)for(let l in s){let a=s[l];a&&n(l).handlers.push(Em(r.value,a))}if(o)for(let l in o){let a=o[l];a&&n(l).observers.push(Em(r.value,a))}}for(let r in Jt)n(r).handlers.push(Jt[r]);for(let r in at)n(r).observers.push(at[r]);return e}const cw=[{key:"Backspace",keyCode:8,inputType:"deleteContentBackward"},{key:"Enter",keyCode:13,inputType:"insertParagraph"},{key:"Enter",keyCode:13,inputType:"insertLineBreak"},{key:"Delete",keyCode:46,inputType:"deleteContentForward"}],WT="dthko",uw=[16,17,18,20,91,92,224,225],nl=6;function rl(t){return Math.max(0,t)*.7+8}function FT(t,e){return Math.max(Math.abs(t.clientX-e.clientX),Math.abs(t.clientY-e.clientY))}class zT{constructor(e,n,r,i){this.view=e,this.startEvent=n,this.style=r,this.mustSelect=i,this.scrollSpeed={x:0,y:0},this.scrolling=-1,this.lastEvent=n,this.scrollParents=Ey(e.contentDOM),this.atoms=e.state.facet(Ao).map(o=>o(e));let s=e.contentDOM.ownerDocument;s.addEventListener("mousemove",this.move=this.move.bind(this)),s.addEventListener("mouseup",this.up=this.up.bind(this)),this.extend=n.shiftKey,this.multiple=e.state.facet(ee.allowMultipleSelections)&&jT(e,n),this.dragging=HT(e,n)&&dw(n)==1?null:!1}start(e){this.dragging===!1&&this.select(e)}move(e){if(e.buttons==0)return this.destroy();if(this.dragging||this.dragging==null&&FT(this.startEvent,e)<10)return;this.select(this.lastEvent=e);let n=0,r=0,i=0,s=0,o=this.view.win.innerWidth,l=this.view.win.innerHeight;this.scrollParents.x&&({left:i,right:o}=this.scrollParents.x.getBoundingClientRect()),this.scrollParents.y&&({top:s,bottom:l}=this.scrollParents.y.getBoundingClientRect());let a=vd(this.view);e.clientX-a.left<=i+nl?n=-rl(i-e.clientX):e.clientX+a.right>=o-nl&&(n=rl(e.clientX-o)),e.clientY-a.top<=s+nl?r=-rl(s-e.clientY):e.clientY+a.bottom>=l-nl&&(r=rl(e.clientY-l)),this.setScrollSpeed(n,r)}up(e){this.dragging==null&&this.select(this.lastEvent),this.dragging||e.preventDefault(),this.destroy()}destroy(){this.setScrollSpeed(0,0);let e=this.view.contentDOM.ownerDocument;e.removeEventListener("mousemove",this.move),e.removeEventListener("mouseup",this.up),this.view.inputState.mouseSelection=this.view.inputState.draggedContent=null}setScrollSpeed(e,n){this.scrollSpeed={x:e,y:n},e||n?this.scrolling<0&&(this.scrolling=setInterval(()=>this.scroll(),50)):this.scrolling>-1&&(clearInterval(this.scrolling),this.scrolling=-1)}scroll(){let{x:e,y:n}=this.scrollSpeed;e&&this.scrollParents.x&&(this.scrollParents.x.scrollLeft+=e,e=0),n&&this.scrollParents.y&&(this.scrollParents.y.scrollTop+=n,n=0),(e||n)&&this.view.win.scrollBy(e,n),this.dragging===!1&&this.select(this.lastEvent)}select(e){let{view:n}=this,r=sw(this.atoms,this.style.get(e,this.extend,this.multiple));(this.mustSelect||!r.eq(n.state.selection,this.dragging===!1))&&this.view.dispatch({selection:r,userEvent:"select.pointer"}),this.mustSelect=!1}update(e){e.transactions.some(n=>n.isUserEvent("input.type"))?this.destroy():this.style.update(e)&&setTimeout(()=>this.select(this.lastEvent),20)}}function jT(t,e){let n=t.state.facet(Vy);return n.length?n[0](e):_.mac?e.metaKey:e.ctrlKey}function VT(t,e){let n=t.state.facet(Hy);return n.length?n[0](e):_.mac?!e.altKey:!e.ctrlKey}function HT(t,e){let{main:n}=t.state.selection;if(n.empty)return!1;let r=uo(t.root);if(!r||r.rangeCount==0)return!0;let i=r.getRangeAt(0).getClientRects();for(let s=0;s<i.length;s++){let o=i[s];if(o.left<=e.clientX&&o.right>=e.clientX&&o.top<=e.clientY&&o.bottom>=e.clientY)return!0}return!1}function UT(t,e){if(!e.bubbles)return!0;if(e.defaultPrevented)return!1;for(let n=e.target,r;n!=t.contentDOM;n=n.parentNode)if(!n||n.nodeType==11||(r=Re.get(n))&&r.isWidget()&&!r.isHidden&&r.widget.ignoreEvent(e))return!1;return!0}const Jt=Object.create(null),at=Object.create(null),hw=_.ie&&_.ie_version<15||_.ios&&_.webkit_version<604;function $T(t){let e=t.dom.parentNode;if(!e)return;let n=e.appendChild(document.createElement("textarea"));n.style.cssText="position: fixed; left: -10000px; top: 10px",n.focus(),setTimeout(()=>{t.focus(),n.remove(),fw(t,n.value)},50)}function nc(t,e,n){for(let r of t.facet(e))n=r(n,t);return n}function fw(t,e){e=nc(t.state,dd,e);let{state:n}=t,r,i=1,s=n.toText(e),o=s.lines==n.selection.ranges.length;if(jh!=null&&n.selection.ranges.every(a=>a.empty)&&jh==s.toString()){let a=-1;r=n.changeByRange(c=>{let u=n.doc.lineAt(c.from);if(u.from==a)return{range:c};a=u.from;let h=n.toText((o?s.line(i++).text:e)+n.lineBreak);return{changes:{from:u.from,insert:h},range:R.cursor(c.from+h.length,-1)}})}else o?r=n.changeByRange(a=>{let c=s.line(i++);return{changes:{from:a.from,to:a.to,insert:c.text},range:R.cursor(a.from+c.length,-1)}}):r=n.replaceSelection(s);t.dispatch(r,{userEvent:"input.paste",scrollIntoView:!0})}at.scroll=t=>{let e=t.inputState;e.lastScrollTop=t.scrollDOM.scrollTop,e.lastScrollLeft=t.scrollDOM.scrollLeft,_.ios&&!e.touchActive&&(e.lastIOSMomentumScroll=Date.now())};at.wheel=at.mousewheel=t=>{t.inputState.lastWheelEvent=Date.now()};Jt.keydown=(t,e)=>(t.inputState.setSelectionOrigin("select"),e.keyCode==27&&t.inputState.tabFocusMode!=0&&(t.inputState.tabFocusMode=Date.now()+2e3),!1);at.touchstart=(t,e)=>{let n=t.inputState,r=e.targetTouches[0];n.touchActive=!0,n.lastTouchTime=Date.now(),r&&(n.lastTouchX=r.clientX,n.lastTouchY=r.clientY),n.setSelectionOrigin("select.pointer")};at.touchmove=t=>{t.inputState.setSelectionOrigin("select.pointer")};at.touchend=(t,e)=>{t.inputState.touchActive=!1};Jt.mousedown=(t,e)=>{if(t.observer.flush(),t.inputState.lastTouchTime>Date.now()-2e3)return!1;let n=null;for(let r of t.state.facet(Uy))if(n=r(t,e),n)break;if(!n&&e.button==0&&(n=KT(t,e)),n){let r=!t.hasFocus;t.inputState.startMouseSelection(new zT(t,e,n,r)),r&&t.observer.ignore(()=>{Oy(t.contentDOM);let s=t.root.activeElement;s&&!s.contains(t.contentDOM)&&s.blur()});let i=t.inputState.mouseSelection;if(i)return i.start(e),i.dragging===!1}else t.inputState.setSelectionOrigin("select.pointer");return!1};function Lm(t,e,n,r){if(r==1)return R.cursor(e,n);if(r==2)return CT(t.state,e,n);{let i=t.docView.lineAt(e,n),s=t.state.doc.lineAt(i?i.posAtEnd:e),o=i?i.posAtStart:s.from,l=i?i.posAtEnd:s.to;return l<t.state.doc.length&&l==s.to&&l++,R.undirectionalRange(o,l)}}const GT=_.ie&&_.ie_version<=11;let Dm=null,Om=0,Nm=0;function dw(t){if(!GT)return t.detail;let e=Dm,n=Nm;return Dm=t,Nm=Date.now(),Om=!e||n>Date.now()-400&&Math.abs(e.clientX-t.clientX)<2&&Math.abs(e.clientY-t.clientY)<2?(Om+1)%3:1}function KT(t,e){let n=t.posAndSideAtCoords({x:e.clientX,y:e.clientY},!1),r=dw(e),i=t.state.selection;return{update(s){s.docChanged&&(n.pos=s.changes.mapPos(n.pos),i=i.map(s.changes))},get(s,o,l){let a=t.posAndSideAtCoords({x:s.clientX,y:s.clientY},!1),c,u=Lm(t,a.pos,a.assoc,r);if(n.pos!=a.pos&&!o){let h=Lm(t,n.pos,n.assoc,r),f=Math.min(h.from,u.from),d=Math.max(h.to,u.to);u=f<u.from?R.range(f,d,u.assoc):R.range(d,f,u.assoc)}return o?i.replaceRange(i.main.extend(u.from,u.to,u.assoc)):l&&r==1&&i.ranges.length>1&&(c=QT(i,a.pos))?c:l?i.addRange(u):R.create([u])}}}function QT(t,e){for(let n=0;n<t.ranges.length;n++){let{from:r,to:i}=t.ranges[n];if(r<=e&&i>=e)return R.create(t.ranges.slice(0,n).concat(t.ranges.slice(n+1)),t.mainIndex==n?0:t.mainIndex-(t.mainIndex>n?1:0))}return null}Jt.dragstart=(t,e)=>{let{selection:{main:n}}=t.state;if(e.target.draggable){let i=t.docView.tile.nearest(e.target);if(i&&i.isWidget()){let s=i.posAtStart,o=s+i.length;(s>=n.to||o<=n.from)&&(n=R.undirectionalRange(s,o))}}let{inputState:r}=t;return r.mouseSelection&&(r.mouseSelection.dragging=!0),r.draggedContent=n,e.dataTransfer&&(e.dataTransfer.setData("Text",nc(t.state,pd,t.state.sliceDoc(n.from,n.to))),e.dataTransfer.effectAllowed="copyMove"),!1};Jt.dragend=t=>(t.inputState.draggedContent=null,!1);function Bm(t,e,n,r){if(n=nc(t.state,dd,n),!n)return;let i=t.posAtCoords({x:e.clientX,y:e.clientY},!1),{draggedContent:s}=t.inputState,o=r&&s&&VT(t,e)?{from:s.from,to:s.to}:null,l={from:i,insert:n},a=t.state.changes(o?[o,l]:l);t.focus(),t.dispatch({changes:a,selection:{anchor:a.mapPos(i,-1),head:a.mapPos(i,1)},userEvent:o?"move.drop":"input.drop"}),t.inputState.draggedContent=null}Jt.drop=(t,e)=>{if(!e.dataTransfer)return!1;if(t.state.readOnly)return!0;let n=e.dataTransfer.files;if(n&&n.length){let r=Array(n.length),i=0,s=()=>{++i==n.length&&Bm(t,e,r.filter(o=>o!=null).join(t.state.lineBreak),!1)};for(let o=0;o<n.length;o++){let l=new FileReader;l.onerror=s,l.onload=()=>{/[\x00-\x08\x0e-\x1f]{2}/.test(l.result)||(r[o]=l.result),s()},l.readAsText(n[o])}return!0}else{let r=e.dataTransfer.getData("Text");if(r)return Bm(t,e,r,!0),!0}return!1};Jt.paste=(t,e)=>{if(t.state.readOnly)return!0;t.observer.flush();let n=hw?null:e.clipboardData;return n?(fw(t,n.getData("text/plain")||n.getData("text/uri-list")),!0):($T(t),!1)};function YT(t,e){let n=t.dom.parentNode;if(!n)return;let r=n.appendChild(document.createElement("textarea"));r.style.cssText="position: fixed; left: -10000px; top: 10px",r.value=e,r.focus(),r.selectionEnd=e.length,r.selectionStart=0,setTimeout(()=>{r.remove(),t.focus()},50)}function XT(t){let e=[],n=[],r=!1;for(let i of t.selection.ranges)i.empty||(e.push(t.sliceDoc(i.from,i.to)),n.push(i));if(!e.length){let i=-1;for(let{from:s}of t.selection.ranges){let o=t.doc.lineAt(s);o.number>i&&(e.push(o.text),n.push({from:o.from,to:Math.min(t.doc.length,o.to+1)})),i=o.number}r=!0}return{text:nc(t,pd,e.join(t.lineBreak)),ranges:n,linewise:r}}let jh=null;Jt.copy=Jt.cut=(t,e)=>{if(!_s(t.contentDOM,t.observer.selectionRange))return!1;let{text:n,ranges:r,linewise:i}=XT(t.state);if(!n&&!i)return!1;jh=i?n:null,e.type=="cut"&&!t.state.readOnly&&t.dispatch({changes:r,scrollIntoView:!0,userEvent:"delete.cut"});let s=hw?null:e.clipboardData;return s?(s.clearData(),s.setData("text/plain",n),!0):(YT(t,n),!1)};const pw=_n.define();function mw(t,e){let n=[];for(let r of t.facet(Ky)){let i=r(t,e);i&&n.push(i)}return n.length?t.update({effects:n,annotations:pw.of(!0)}):null}function gw(t){setTimeout(()=>{let e=t.hasFocus;if(e!=t.inputState.notifiedFocused){let n=mw(t.state,e);n?t.dispatch(n):t.update([])}},10)}at.focus=t=>{t.inputState.lastFocusTime=Date.now(),!t.scrollDOM.scrollTop&&(t.inputState.lastScrollTop||t.inputState.lastScrollLeft)&&(t.scrollDOM.scrollTop=t.inputState.lastScrollTop,t.scrollDOM.scrollLeft=t.inputState.lastScrollLeft),gw(t)};at.blur=t=>{t.observer.clearSelectionRange(),gw(t)};at.compositionstart=at.compositionupdate=(t,e)=>{if(!t.observer.editContext&&(t.inputState.compositionFirstChange==null&&(t.inputState.compositionFirstChange=!0),t.inputState.composing<0)){let{main:n}=t.state.selection;!n.empty&&t.lineBlockAt(n.from).from!=t.lineBlockAt(n.to).from&&!t.state.readOnly&&t.dispatch({changes:t.state.selection.ranges.filter(r=>!r.empty).map(r=>({from:r.from,to:r.to})),userEvent:"input"}),t.inputState.composing=0}};at.compositionend=t=>{t.observer.editContext||(t.inputState.composing=-1,t.inputState.compositionEndedAt=Date.now(),t.inputState.compositionPendingKey=!0,t.inputState.compositionPendingChange=t.observer.pendingRecords().length>0,t.inputState.compositionFirstChange=null,_.chrome&&_.android?t.observer.flushSoon():t.inputState.compositionPendingChange?Promise.resolve().then(()=>t.observer.flush()):setTimeout(()=>{t.inputState.composing<0&&t.docView.hasComposition&&t.update([])},50))};at.contextmenu=t=>{t.inputState.lastContextMenu=Date.now()};Jt.beforeinput=(t,e)=>{var n,r;if((e.inputType=="insertText"||e.inputType=="insertCompositionText")&&(t.inputState.insertingText=e.data,t.inputState.insertingTextAt=Date.now()),e.inputType=="insertReplacementText"&&t.observer.editContext){let s=(n=e.dataTransfer)===null||n===void 0?void 0:n.getData("text/plain"),o=e.getTargetRanges();if(s&&o.length){let l=o[0],a=t.posAtDOM(l.startContainer,l.startOffset),c=t.posAtDOM(l.endContainer,l.endOffset);return yd(t,{from:a,to:c,insert:t.state.toText(s)},null),!0}}let i;if(_.chrome&&_.android&&(i=cw.find(s=>s.inputType==e.inputType))&&(t.observer.delayAndroidKey(i.key,i.keyCode),i.key=="Backspace"||i.key=="Delete")){let s=((r=window.visualViewport)===null||r===void 0?void 0:r.height)||0;setTimeout(()=>{var o;(((o=window.visualViewport)===null||o===void 0?void 0:o.height)||0)>s+10&&t.hasFocus&&(t.contentDOM.blur(),t.focus())},100)}return _.ios&&e.inputType=="deleteContentForward"&&t.observer.flushSoon(),_.safari&&e.inputType=="insertText"&&t.inputState.composing>=0&&setTimeout(()=>at.compositionend(t,e),20),!1};const _m=new Set;function JT(t){_m.has(t)||(_m.add(t),t.addEventListener("copy",()=>{}),t.addEventListener("cut",()=>{}))}const qm=["pre-wrap","normal","pre-line","break-spaces"];let $i=!1;function Wm(){$i=!1}class ZT{constructor(e){this.lineWrapping=e,this.doc=te.empty,this.heightSamples={},this.lineHeight=14,this.charWidth=7,this.textHeight=14,this.lineLength=30}heightForGap(e,n){let r=this.doc.lineAt(n).number-this.doc.lineAt(e).number+1;return this.lineWrapping&&(r+=Math.max(0,Math.ceil((n-e-r*this.lineLength*.5)/this.lineLength))),this.lineHeight*r}heightForLine(e){return this.lineWrapping?(1+Math.max(0,Math.ceil((e-this.lineLength)/Math.max(1,this.lineLength-5))))*this.lineHeight:this.lineHeight}setDoc(e){return this.doc=e,this}mustRefreshForWrapping(e){return qm.indexOf(e)>-1!=this.lineWrapping}mustRefreshForHeights(e){let n=!1;for(let r=0;r<e.length;r++){let i=e[r];i<0?r++:this.heightSamples[Math.floor(i*10)]||(n=!0,this.heightSamples[Math.floor(i*10)]=!0)}return n}refresh(e,n,r,i,s,o){let l=qm.indexOf(e)>-1,a=Math.abs(n-this.lineHeight)>.3||this.lineWrapping!=l;if(this.lineWrapping=l,this.lineHeight=n,this.charWidth=r,this.textHeight=i,this.lineLength=s,a){this.heightSamples={};for(let c=0;c<o.length;c++){let u=o[c];u<0?c++:this.heightSamples[Math.floor(u*10)]=!0}}return a}}class eR{constructor(e,n){this.from=e,this.heights=n,this.index=0}get more(){return this.index<this.heights.length}}class $t{constructor(e,n,r,i,s){this.from=e,this.length=n,this.top=r,this.height=i,this._content=s}get type(){return typeof this._content=="number"?Ge.Text:Array.isArray(this._content)?this._content:this._content.type}get to(){return this.from+this.length}get bottom(){return this.top+this.height}get widget(){return this._content instanceof Ur?this._content.widget:null}get widgetLineBreaks(){return typeof this._content=="number"?this._content:0}join(e){let n=(Array.isArray(this._content)?this._content:[this]).concat(Array.isArray(e._content)?e._content:[e]);return new $t(this.from,this.length+e.length,this.top,this.height+e.height,n)}}var he=function(t){return t[t.ByPos=0]="ByPos",t[t.ByHeight=1]="ByHeight",t[t.ByPosNoHeight=2]="ByPosNoHeight",t}(he||(he={}));const Bl=.001;class lt{constructor(e,n,r=2){this.length=e,this.height=n,this.flags=r}get outdated(){return(this.flags&2)>0}set outdated(e){this.flags=(e?2:0)|this.flags&-3}setHeight(e){this.height!=e&&(Math.abs(this.height-e)>Bl&&($i=!0),this.height=e)}replace(e,n,r){return lt.of(r)}decomposeLeft(e,n){n.push(this)}decomposeRight(e,n){n.push(this)}applyChanges(e,n,r,i){let s=this,o=r.doc;for(let l=i.length-1;l>=0;l--){let{fromA:a,toA:c,fromB:u,toB:h}=i[l],f=s.lineAt(a,he.ByPosNoHeight,r.setDoc(n),0,0),d=f.to>=c?f:s.lineAt(c,he.ByPosNoHeight,r,0,0);for(h+=d.to-c,c=d.to;l>0&&f.from<=i[l-1].toA;)a=i[l-1].fromA,u=i[l-1].fromB,l--,a<f.from&&(f=s.lineAt(a,he.ByPosNoHeight,r,0,0));u+=f.from-a,a=f.from;let p=wd.build(r.setDoc(o),e,u,h);s=ka(s,s.replace(a,c,p))}return s.updateHeight(r,0)}static empty(){return new Ct(0,0,0)}static of(e){if(e.length==1)return e[0];let n=0,r=e.length,i=0,s=0;for(;;)if(n==r)if(i>s*2){let l=e[n-1];l.break?e.splice(--n,1,l.left,null,l.right):e.splice(--n,1,l.left,l.right),r+=1+l.break,i-=l.size}else if(s>i*2){let l=e[r];l.break?e.splice(r,1,l.left,null,l.right):e.splice(r,1,l.left,l.right),r+=2+l.break,s-=l.size}else break;else if(i<s){let l=e[n++];l&&(i+=l.size)}else{let l=e[--r];l&&(s+=l.size)}let o=!1;return e[n-1]==null?(o=!0,n--):e[n]==null&&(o=!0,r++),new nR(lt.of(e.slice(0,n)),o,lt.of(e.slice(r)))}}function ka(t,e){return t==e?t:(t.constructor!=e.constructor&&($i=!0),e)}lt.prototype.size=1;const tR=U.replace({});class vw extends lt{constructor(e,n,r){super(e,n),this.deco=r,this.spaceAbove=0}mainBlock(e,n){return new $t(n,this.length,e+this.spaceAbove,this.height-this.spaceAbove,this.deco||0)}blockAt(e,n,r,i){return this.spaceAbove&&e<r+this.spaceAbove?new $t(i,0,r,this.spaceAbove,tR):this.mainBlock(r,i)}lineAt(e,n,r,i,s){let o=this.mainBlock(i,s);return this.spaceAbove?this.blockAt(0,r,i,s).join(o):o}forEachLine(e,n,r,i,s,o){e<=s+this.length&&n>=s&&o(this.lineAt(0,he.ByPos,r,i,s))}setMeasuredHeight(e){let n=e.heights[e.index++];n<0?(this.spaceAbove=-n,n=e.heights[e.index++]):this.spaceAbove=0,this.setHeight(n)}updateHeight(e,n=0,r=!1,i){return i&&i.from<=n&&i.more&&this.setMeasuredHeight(i),this.outdated=!1,this}toString(){return`block(${this.length})`}}class Ct extends vw{constructor(e,n,r){super(e,n,null),this.collapsed=0,this.widgetHeight=0,this.breaks=0,this.spaceAbove=r}mainBlock(e,n){return new $t(n,this.length,e+this.spaceAbove,this.height-this.spaceAbove,this.breaks)}replace(e,n,r){let i=r[0];return r.length==1&&(i instanceof Ct||i instanceof He&&i.flags&4)&&Math.abs(this.length-i.length)<10?(i instanceof He?i=new Ct(i.length,this.height,this.spaceAbove):i.height=this.height,this.outdated||(i.outdated=!1),i):lt.of(r)}updateHeight(e,n=0,r=!1,i){return i&&i.from<=n&&i.more?this.setMeasuredHeight(i):(r||this.outdated)&&(this.spaceAbove=0,this.setHeight(Math.max(this.widgetHeight,e.heightForLine(this.length-this.collapsed))+this.breaks*e.lineHeight)),this.outdated=!1,this}toString(){return`line(${this.length}${this.collapsed?-this.collapsed:""}${this.widgetHeight?":"+this.widgetHeight:""})`}}class He extends lt{constructor(e){super(e,0)}heightMetrics(e,n){let r=e.doc.lineAt(n).number,i=e.doc.lineAt(n+this.length).number,s=i-r+1,o,l=0;if(e.lineWrapping){let a=Math.min(this.height,e.lineHeight*s);o=a/s,this.length>s+1&&(l=(this.height-a)/(this.length-s-1))}else o=this.height/s;return{firstLine:r,lastLine:i,perLine:o,perChar:l}}blockAt(e,n,r,i){let{firstLine:s,lastLine:o,perLine:l,perChar:a}=this.heightMetrics(n,i);if(n.lineWrapping){let c=i+(e<n.lineHeight?0:Math.round(Math.max(0,Math.min(1,(e-r)/this.height))*this.length)),u=n.doc.lineAt(c),h=l+u.length*a,f=Math.max(r,e-h/2);return new $t(u.from,u.length,f,h,0)}else{let c=Math.max(0,Math.min(o-s,Math.floor((e-r)/l))),{from:u,length:h}=n.doc.line(s+c);return new $t(u,h,r+l*c,l,0)}}lineAt(e,n,r,i,s){if(n==he.ByHeight)return this.blockAt(e,r,i,s);if(n==he.ByPosNoHeight){let{from:d,to:p}=r.doc.lineAt(e);return new $t(d,p-d,0,0,0)}let{firstLine:o,perLine:l,perChar:a}=this.heightMetrics(r,s),c=r.doc.lineAt(e),u=l+c.length*a,h=c.number-o,f=i+l*h+a*(c.from-s-h);return new $t(c.from,c.length,Math.max(i,Math.min(f,i+this.height-u)),u,0)}forEachLine(e,n,r,i,s,o){e=Math.max(e,s),n=Math.min(n,s+this.length);let{firstLine:l,perLine:a,perChar:c}=this.heightMetrics(r,s);for(let u=e,h=i;u<=n;){let f=r.doc.lineAt(u);if(u==e){let p=f.number-l;h+=a*p+c*(e-s-p)}let d=a+c*f.length;o(new $t(f.from,f.length,h,d,0)),h+=d,u=f.to+1}}replace(e,n,r){let i=this.length-n;if(i>0){let s=r[r.length-1];s instanceof He?r[r.length-1]=new He(s.length+i):r.push(null,new He(i-1))}if(e>0){let s=r[0];s instanceof He?r[0]=new He(e+s.length):r.unshift(new He(e-1),null)}return lt.of(r)}decomposeLeft(e,n){n.push(new He(e-1),null)}decomposeRight(e,n){n.push(null,new He(this.length-e-1))}updateHeight(e,n=0,r=!1,i){let s=n+this.length;if(i&&i.from<=n+this.length&&i.more){let o=[],l=Math.max(n,i.from),a=-1;for(i.from>n&&o.push(new He(i.from-n-1).updateHeight(e,n));l<=s&&i.more;){let u=e.doc.lineAt(l).length;o.length&&o.push(null);let h=i.heights[i.index++],f=0;h<0&&(f=-h,h=i.heights[i.index++]),a==-1?a=h:Math.abs(h-a)>=Bl&&(a=-2);let d=new Ct(u,h,f);d.outdated=!1,o.push(d),l+=u+1}l<=s&&o.push(null,new He(s-l).updateHeight(e,l));let c=lt.of(o);return(a<0||Math.abs(c.height-this.height)>=Bl||Math.abs(a-this.heightMetrics(e,n).perLine)>=Bl)&&($i=!0),ka(this,c)}else(r||this.outdated)&&(this.setHeight(e.heightForGap(n,n+this.length)),this.outdated=!1);return this}toString(){return`gap(${this.length})`}}class nR extends lt{constructor(e,n,r){super(e.length+(n?1:0)+r.length,e.height+r.height,(n?1:0)|(e.outdated||r.outdated?2:0)),this.left=e,this.right=r,this.size=e.size+r.size}get break(){return this.flags&1}blockAt(e,n,r,i){let s=r+this.left.height;return e<s?this.left.blockAt(e,n,r,i):this.right.blockAt(e,n,s,i+this.left.length+this.break)}lineAt(e,n,r,i,s){let o=i+this.left.height,l=s+this.left.length+this.break,a=n==he.ByHeight?e<o:e<l,c=a?this.left.lineAt(e,n,r,i,s):this.right.lineAt(e,n,r,o,l);if(this.break||(a?c.to<l:c.from>l))return c;let u=n==he.ByPosNoHeight?he.ByPosNoHeight:he.ByPos;return a?c.join(this.right.lineAt(l,u,r,o,l)):this.left.lineAt(l,u,r,i,s).join(c)}forEachLine(e,n,r,i,s,o){let l=i+this.left.height,a=s+this.left.length+this.break;if(this.break)e<a&&this.left.forEachLine(e,n,r,i,s,o),n>=a&&this.right.forEachLine(e,n,r,l,a,o);else{let c=this.lineAt(a,he.ByPos,r,i,s);e<c.from&&this.left.forEachLine(e,Math.min(n,c.from-1),r,i,s,o),c.to>=e&&c.from<=n&&o(c),n>c.to&&this.right.forEachLine(Math.max(e,c.to+1),n,r,l,a,o)}}replace(e,n,r){let i=this.left.length+this.break;if(n<i)return this.balanced(this.left.replace(e,n,r),this.right);if(e>this.left.length)return this.balanced(this.left,this.right.replace(e-i,n-i,r));let s=[];e>0&&this.decomposeLeft(e,s);let o=s.length;for(let l of r)s.push(l);if(e>0&&Fm(s,o-1),n<this.length){let l=s.length;this.decomposeRight(n,s),Fm(s,l)}return lt.of(s)}decomposeLeft(e,n){let r=this.left.length;if(e<=r)return this.left.decomposeLeft(e,n);n.push(this.left),this.break&&(r++,e>=r&&n.push(null)),e>r&&this.right.decomposeLeft(e-r,n)}decomposeRight(e,n){let r=this.left.length,i=r+this.break;if(e>=i)return this.right.decomposeRight(e-i,n);e<r&&this.left.decomposeRight(e,n),this.break&&e<i&&n.push(null),n.push(this.right)}balanced(e,n){return e.size>2*n.size||n.size>2*e.size?lt.of(this.break?[e,null,n]:[e,n]):(this.left=ka(this.left,e),this.right=ka(this.right,n),this.setHeight(e.height+n.height),this.outdated=e.outdated||n.outdated,this.size=e.size+n.size,this.length=e.length+this.break+n.length,this)}updateHeight(e,n=0,r=!1,i){let{left:s,right:o}=this,l=n+s.length+this.break,a=null;return i&&i.from<=n+s.length&&i.more?a=s=s.updateHeight(e,n,r,i):s.updateHeight(e,n,r),i&&i.from<=l+o.length&&i.more?a=o=o.updateHeight(e,l,r,i):o.updateHeight(e,l,r),a?this.balanced(s,o):(this.height=this.left.height+this.right.height,this.outdated=!1,this)}toString(){return this.left+(this.break?" ":"-")+this.right}}function Fm(t,e){let n,r;t[e]==null&&(n=t[e-1])instanceof He&&(r=t[e+1])instanceof He&&t.splice(e-1,3,new He(n.length+1+r.length))}const rR=5;class wd{constructor(e,n){this.pos=e,this.oracle=n,this.nodes=[],this.lineStart=-1,this.lineEnd=-1,this.covering=null,this.writtenTo=e}get isCovered(){return this.covering&&this.nodes[this.nodes.length-1]==this.covering}span(e,n){if(this.lineStart>-1){let r=Math.min(n,this.lineEnd),i=this.nodes[this.nodes.length-1];i instanceof Ct?i.length+=r-this.pos:(r>this.pos||!this.isCovered)&&this.nodes.push(new Ct(r-this.pos,-1,0)),this.writtenTo=r,n>r&&(this.nodes.push(null),this.writtenTo++,this.lineStart=-1)}this.pos=n}point(e,n,r){if(e<n||r.heightRelevant){let i=r.widget?r.widget.estimatedHeight:0,s=r.widget?r.widget.lineBreaks:0;i<0&&(i=this.oracle.lineHeight);let o=n-e;r.block?this.addBlock(new vw(o,i,r)):(o||s||i>=rR)&&this.addLineDeco(i,s,o)}else n>e&&this.span(e,n);this.lineEnd>-1&&this.lineEnd<this.pos&&(this.lineEnd=this.oracle.doc.lineAt(this.pos).to)}enterLine(){if(this.lineStart>-1)return;let{from:e,to:n}=this.oracle.doc.lineAt(this.pos);this.lineStart=e,this.lineEnd=n,this.writtenTo<e&&((this.writtenTo<e-1||this.nodes[this.nodes.length-1]==null)&&this.nodes.push(this.blankContent(this.writtenTo,e-1)),this.nodes.push(null)),this.pos>e&&this.nodes.push(new Ct(this.pos-e,-1,0)),this.writtenTo=this.pos}blankContent(e,n){let r=new He(n-e);return this.oracle.doc.lineAt(e).to==n&&(r.flags|=4),r}ensureLine(){this.enterLine();let e=this.nodes.length?this.nodes[this.nodes.length-1]:null;if(e instanceof Ct)return e;let n=new Ct(0,-1,0);return this.nodes.push(n),n}addBlock(e){this.enterLine();let n=e.deco;n&&n.startSide>0&&!this.isCovered&&this.ensureLine(),this.nodes.push(e),this.writtenTo=this.pos=this.pos+e.length,n&&n.endSide>0&&(this.covering=e)}addLineDeco(e,n,r){let i=this.ensureLine();i.length+=r,i.collapsed+=r,i.widgetHeight=Math.max(i.widgetHeight,e),i.breaks+=n,this.writtenTo=this.pos=this.pos+r}finish(e){let n=this.nodes.length==0?null:this.nodes[this.nodes.length-1];this.lineStart>-1&&!(n instanceof Ct)&&!this.isCovered?this.nodes.push(new Ct(0,-1,0)):(this.writtenTo<this.pos||n==null)&&this.nodes.push(this.blankContent(this.writtenTo,this.pos));let r=e;for(let i of this.nodes)i instanceof Ct&&i.updateHeight(this.oracle,r),r+=i?i.length:1;return this.nodes}static build(e,n,r,i){let s=new wd(r,e);return X.spans(n,r,i,s,0),s.finish(r)}}function iR(t,e,n){let r=new sR;return X.compare(t,e,n,r,0),r.changes}class sR{constructor(){this.changes=[]}compareRange(){}comparePoint(e,n,r,i){(e<n||r&&r.heightRelevant||i&&i.heightRelevant)&&Ai(e,n,this.changes,5)}}function oR(t,e){let n=t.getBoundingClientRect(),r=t.ownerDocument,i=r.defaultView||window,s=Math.max(0,n.left),o=Math.min(i.innerWidth,n.right),l=Math.max(0,n.top),a=Math.min(i.innerHeight,n.bottom);for(let c=t.parentNode;c&&c!=r.body;)if(Fe(c)){let u=window.getComputedStyle(c);if((c.scrollHeight>c.clientHeight||c.scrollWidth>c.clientWidth)&&u.overflow!="visible"){let h=c.getBoundingClientRect();s=Math.max(s,h.left),o=Math.min(o,h.right),l=Math.max(l,h.top),a=Math.min(c==t.parentNode?i.innerHeight:a,h.bottom)}c=u.position=="absolute"||u.position=="fixed"?c.offsetParent:c.parentNode}else if(c.nodeType==11)c=c.host;else break;return{left:s-n.left,right:Math.max(s,o)-n.left,top:l-(n.top+e),bottom:Math.max(l,a)-(n.top+e)}}function lR(t){let e=t.getBoundingClientRect(),n=t.ownerDocument.defaultView||window;return e.left<n.innerWidth&&e.right>0&&e.top<n.innerHeight&&e.bottom>0}function aR(t,e){let n=t.getBoundingClientRect();return{left:0,right:n.right-n.left,top:e,bottom:n.bottom-(n.top+e)}}class Zc{constructor(e,n,r,i){this.from=e,this.to=n,this.size=r,this.displaySize=i}static same(e,n){if(e.length!=n.length)return!1;for(let r=0;r<e.length;r++){let i=e[r],s=n[r];if(i.from!=s.from||i.to!=s.to||i.size!=s.size)return!1}return!0}draw(e,n){return U.replace({widget:new cR(this.displaySize*(n?e.scaleY:e.scaleX),n)}).range(this.from,this.to)}}class cR extends qn{constructor(e,n){super(),this.size=e,this.vertical=n}eq(e){return e.size==this.size&&e.vertical==this.vertical}toDOM(){let e=document.createElement("div");return this.vertical?e.style.height=this.size+"px":(e.style.width=this.size+"px",e.style.height="2px",e.style.display="inline-block"),e}get estimatedHeight(){return this.vertical?this.size:-1}}class zm{constructor(e,n){this.view=e,this.state=n,this.pixelViewport={left:0,right:window.innerWidth,top:0,bottom:0},this.inView=!0,this.paddingTop=0,this.paddingBottom=0,this.contentDOMWidth=0,this.contentDOMHeight=0,this.editorHeight=0,this.editorWidth=0,this.scaleX=1,this.scaleY=1,this.scrollOffset=0,this.scrolledToBottom=!1,this.scrollAnchorPos=0,this.scrollAnchorHeight=-1,this.scaler=jm,this.scrollTarget=null,this.printing=!1,this.mustMeasureContent=!0,this.defaultTextDirection=ue.LTR,this.visibleRanges=[],this.mustEnforceCursorAssoc=!1;let r=n.facet(md).some(i=>typeof i!="function"&&i.class=="cm-lineWrapping");this.heightOracle=new ZT(r),this.stateDeco=Vm(n),this.heightMap=lt.empty().applyChanges(this.stateDeco,te.empty,this.heightOracle.setDoc(n.doc),[new Ot(0,0,0,n.doc.length)]);for(let i=0;i<2&&(this.viewport=this.getViewport(0,null),!!this.updateForViewport());i++);this.updateViewportLines(),this.lineGaps=this.ensureLineGaps([]),this.lineGapDeco=U.set(this.lineGaps.map(i=>i.draw(this,!1))),this.scrollParent=e.scrollDOM,this.computeVisibleRanges()}updateForViewport(){let e=[this.viewport],{main:n}=this.state.selection;for(let r=0;r<=1;r++){let i=r?n.head:n.anchor;if(!e.some(({from:s,to:o})=>i>=s&&i<=o)){let{from:s,to:o}=this.lineBlockAt(i);e.push(new il(s,o))}}return this.viewports=e.sort((r,i)=>r.from-i.from),this.updateScaler()}updateScaler(){let e=this.scaler;return this.scaler=this.heightMap.height<=7e6?jm:new Sd(this.heightOracle,this.heightMap,this.viewports),e.eq(this.scaler)?0:2}updateViewportLines(){this.viewportLines=[],this.heightMap.forEachLine(this.viewport.from,this.viewport.to,this.heightOracle.setDoc(this.state.doc),0,0,e=>{this.viewportLines.push(bs(e,this.scaler))})}update(e,n=null){this.state=e.state;let r=this.stateDeco;this.stateDeco=Vm(this.state);let i=e.changedRanges,s=Ot.extendWithRanges(i,iR(r,this.stateDeco,e?e.changes:De.empty(this.state.doc.length))),o=this.heightMap.height,l=this.scrolledToBottom?null:this.scrollAnchorAt(this.scrollOffset);Wm(),this.heightMap=this.heightMap.applyChanges(this.stateDeco,e.startState.doc,this.heightOracle.setDoc(this.state.doc),s),(this.heightMap.height!=o||$i)&&(e.flags|=2),l?(this.scrollAnchorPos=e.changes.mapPos(l.from,-1),this.scrollAnchorHeight=l.top):(this.scrollAnchorPos=-1,this.scrollAnchorHeight=o);let a=s.length?this.mapViewport(this.viewport,e.changes):this.viewport;(n&&(n.range.head<a.from||n.range.head>a.to)||!this.viewportIsAppropriate(a))&&(a=this.getViewport(0,n));let c=a.from!=this.viewport.from||a.to!=this.viewport.to;this.viewport=a,e.flags|=this.updateForViewport(),(c||!e.changes.empty||e.flags&2)&&this.updateViewportLines(),(this.lineGaps.length||this.viewport.to-this.viewport.from>4e3)&&this.updateLineGaps(this.ensureLineGaps(this.mapLineGaps(this.lineGaps,e.changes))),e.flags|=this.computeVisibleRanges(e.changes),n&&(this.scrollTarget=n),!this.mustEnforceCursorAssoc&&(e.selectionSet||e.focusChanged)&&e.view.lineWrapping&&e.state.selection.main.empty&&e.state.selection.main.assoc&&!e.state.facet(Yy)&&(this.mustEnforceCursorAssoc=!0)}measure(){let{view:e}=this,n=e.contentDOM,r=window.getComputedStyle(n),i=this.heightOracle,s=r.whiteSpace;this.defaultTextDirection=r.direction=="rtl"?ue.RTL:ue.LTR;let o=this.heightOracle.mustRefreshForWrapping(s)||this.mustMeasureContent==="refresh",l=n.getBoundingClientRect(),a=o||this.mustMeasureContent||this.contentDOMHeight!=l.height;this.contentDOMHeight=l.height,this.mustMeasureContent=!1;let c=0,u=0;if(l.width&&l.height){let{scaleX:k,scaleY:C}=Py(n,l);(k>.005&&Math.abs(this.scaleX-k)>.005||C>.005&&Math.abs(this.scaleY-C)>.005)&&(this.scaleX=k,this.scaleY=C,c|=16,o=a=!0)}let h=(parseInt(r.paddingTop)||0)*this.scaleY,f=(parseInt(r.paddingBottom)||0)*this.scaleY;(this.paddingTop!=h||this.paddingBottom!=f)&&(this.paddingTop=h,this.paddingBottom=f,c|=18),this.editorWidth!=e.scrollDOM.clientWidth&&(i.lineWrapping&&(a=!0),this.editorWidth=e.scrollDOM.clientWidth,c|=16);let d=Ey(this.view.contentDOM,!1).y;d!=this.scrollParent&&(this.scrollParent=d,this.scrollAnchorHeight=-1,this.scrollOffset=0);let p=this.getScrollOffset();this.scrollOffset!=p&&(this.scrollAnchorHeight=-1,this.scrollOffset=p),this.scrolledToBottom=Ny(this.scrollParent||e.win);let m=(this.printing?aR:oR)(n,this.paddingTop),w=m.top-this.pixelViewport.top,g=m.bottom-this.pixelViewport.bottom;this.pixelViewport=m;let v=this.pixelViewport.bottom>this.pixelViewport.top&&this.pixelViewport.right>this.pixelViewport.left;if(v!=this.inView&&(this.inView=v,v&&(a=!0)),!this.inView&&!this.scrollTarget&&!lR(e.dom))return 0;let y=l.width;if((this.contentDOMWidth!=y||this.editorHeight!=e.scrollDOM.clientHeight)&&(this.contentDOMWidth=l.width,this.editorHeight=e.scrollDOM.clientHeight,c|=16),a){let k=e.docView.measureVisibleLineHeights(this.viewport);if(i.mustRefreshForHeights(k)&&(o=!0),o||i.lineWrapping&&Math.abs(y-this.contentDOMWidth)>i.charWidth){let{lineHeight:C,charWidth:b,textHeight:I}=e.docView.measureTextSize();o=C>0&&i.refresh(s,C,b,I,Math.max(5,y/b),k),o&&(e.docView.minWidth=0,c|=16)}w>0&&g>0?u=Math.max(w,g):w<0&&g<0&&(u=Math.min(w,g)),Wm();for(let C of this.viewports){let b=C.from==this.viewport.from?k:e.docView.measureVisibleLineHeights(C);this.heightMap=(o?lt.empty().applyChanges(this.stateDeco,te.empty,this.heightOracle,[new Ot(0,0,0,e.state.doc.length)]):this.heightMap).updateHeight(i,0,o,new eR(C.from,b))}$i&&(c|=2)}let x=!this.viewportIsAppropriate(this.viewport,u)||this.scrollTarget&&(this.scrollTarget.range.head<this.viewport.from||this.scrollTarget.range.head>this.viewport.to);return x&&(c&2&&(c|=this.updateScaler()),this.viewport=this.getViewport(u,this.scrollTarget),c|=this.updateForViewport()),(c&2||x)&&this.updateViewportLines(),(this.lineGaps.length||this.viewport.to-this.viewport.from>4e3)&&this.updateLineGaps(this.ensureLineGaps(o?[]:this.lineGaps,e)),c|=this.computeVisibleRanges(),this.mustEnforceCursorAssoc&&(this.mustEnforceCursorAssoc=!1,e.docView.enforceCursorAssoc()),c}get visibleTop(){return this.scaler.fromDOM(this.pixelViewport.top)}get visibleBottom(){return this.scaler.fromDOM(this.pixelViewport.bottom)}getViewport(e,n){let r=.5-Math.max(-.5,Math.min(.5,e/1e3/2)),i=this.heightMap,s=this.heightOracle,{visibleTop:o,visibleBottom:l}=this,a=new il(i.lineAt(o-r*1e3,he.ByHeight,s,0,0).from,i.lineAt(l+(1-r)*1e3,he.ByHeight,s,0,0).to);if(n){let{head:c}=n.range;if(c<a.from||c>a.to){let u=Math.min(this.editorHeight,this.pixelViewport.bottom-this.pixelViewport.top),h=i.lineAt(c,he.ByPos,s,0,0),f;n.y=="center"?f=(h.top+h.bottom)/2-u/2:n.y=="start"||n.y=="nearest"&&c<a.from?f=h.top:f=h.bottom-u,a=new il(i.lineAt(f-1e3/2,he.ByHeight,s,0,0).from,i.lineAt(f+u+1e3/2,he.ByHeight,s,0,0).to)}}return a}mapViewport(e,n){let r=n.mapPos(e.from,-1),i=n.mapPos(e.to,1);return new il(this.heightMap.lineAt(r,he.ByPos,this.heightOracle,0,0).from,this.heightMap.lineAt(i,he.ByPos,this.heightOracle,0,0).to)}viewportIsAppropriate({from:e,to:n},r=0){if(!this.inView)return!0;let{top:i}=this.heightMap.lineAt(e,he.ByPos,this.heightOracle,0,0),{bottom:s}=this.heightMap.lineAt(n,he.ByPos,this.heightOracle,0,0),{visibleTop:o,visibleBottom:l}=this;return(e==0||i<=o-Math.max(10,Math.min(-r,250)))&&(n==this.state.doc.length||s>=l+Math.max(10,Math.min(r,250)))&&i>o-2*1e3&&s<l+2*1e3}mapLineGaps(e,n){if(!e.length||n.empty)return e;let r=[];for(let i of e)n.touchesRange(i.from,i.to)||r.push(new Zc(n.mapPos(i.from),n.mapPos(i.to),i.size,i.displaySize));return r}ensureLineGaps(e,n){let r=this.heightOracle.lineWrapping,i=r?1e4:2e3,s=i>>1,o=i<<1;if(this.defaultTextDirection!=ue.LTR&&!r)return[];let l=[],a=(u,h,f,d)=>{if(h-u<s)return;let p=this.state.selection.main,m=[p.from];p.empty||m.push(p.to);for(let g of m)if(g>u&&g<h){a(u,g-10,f,d),a(g+10,h,f,d);return}let w=hR(e,g=>g.from>=f.from&&g.to<=f.to&&Math.abs(g.from-u)<s&&Math.abs(g.to-h)<s&&!m.some(v=>g.from<v&&g.to>v));if(!w){if(h<f.to&&n&&r&&n.visibleRanges.some(y=>y.from<=h&&y.to>=h)){let y=n.moveToLineBoundary(R.cursor(h),!1,!0).head;y>u&&(h=y)}let g=this.gapSize(f,u,h,d),v=r||g<2e6?g:2e6;w=new Zc(u,h,g,v)}l.push(w)},c=u=>{if(u.length<o||u.type!=Ge.Text)return;let h=uR(u.from,u.to,this.stateDeco);if(h.total<o)return;let f=this.scrollTarget?this.scrollTarget.range.head:null,d,p;if(r){let m=i/this.heightOracle.lineLength*this.heightOracle.lineHeight,w,g;if(f!=null){let v=ol(h,f),y=((this.visibleBottom-this.visibleTop)/2+m)/u.height;w=v-y,g=v+y}else w=(this.visibleTop-u.top-m)/u.height,g=(this.visibleBottom-u.top+m)/u.height;d=sl(h,w),p=sl(h,g)}else{let m=h.total*this.heightOracle.charWidth,w=i*this.heightOracle.charWidth,g=0;if(m>2e6)for(let C of e)C.from>=u.from&&C.from<u.to&&C.size!=C.displaySize&&C.from*this.heightOracle.charWidth+g<this.pixelViewport.left&&(g=C.size-C.displaySize);let v=this.pixelViewport.left+g,y=this.pixelViewport.right+g,x,k;if(f!=null){let C=ol(h,f),b=((y-v)/2+w)/m;x=C-b,k=C+b}else x=(v-w)/m,k=(y+w)/m;d=sl(h,x),p=sl(h,k)}d>u.from&&a(u.from,d,u,h),p<u.to&&a(p,u.to,u,h)};for(let u of this.viewportLines)Array.isArray(u.type)?u.type.forEach(c):c(u);return l}gapSize(e,n,r,i){let s=ol(i,r)-ol(i,n);return this.heightOracle.lineWrapping?e.height*s:i.total*this.heightOracle.charWidth*s}updateLineGaps(e){Zc.same(e,this.lineGaps)||(this.lineGaps=e,this.lineGapDeco=U.set(e.map(n=>n.draw(this,this.heightOracle.lineWrapping))))}computeVisibleRanges(e){let n=this.stateDeco;this.lineGaps.length&&(n=n.concat(this.lineGapDeco));let r=[];X.spans(n,this.viewport.from,this.viewport.to,{span(s,o){r.push({from:s,to:o})},point(){}},20);let i=0;if(r.length!=this.visibleRanges.length)i=12;else for(let s=0;s<r.length&&!(i&8);s++){let o=this.visibleRanges[s],l=r[s];(o.from!=l.from||o.to!=l.to)&&(i|=4,e&&e.mapPos(o.from,-1)==l.from&&e.mapPos(o.to,1)==l.to||(i|=8))}return this.visibleRanges=r,i}lineBlockAt(e){return e>=this.viewport.from&&e<=this.viewport.to&&this.viewportLines.find(n=>n.from<=e&&n.to>=e)||bs(this.heightMap.lineAt(e,he.ByPos,this.heightOracle,0,0),this.scaler)}lineBlockAtHeight(e){return e>=this.viewportLines[0].top&&e<=this.viewportLines[this.viewportLines.length-1].bottom&&this.viewportLines.find(n=>n.top<=e&&n.bottom>=e)||bs(this.heightMap.lineAt(this.scaler.fromDOM(e),he.ByHeight,this.heightOracle,0,0),this.scaler)}getScrollOffset(){return this.scrollParent==this.view.scrollDOM?this.scrollParent.scrollTop*this.scaleY:(this.scrollParent?this.scrollParent.getBoundingClientRect().top:0)-this.view.contentDOM.getBoundingClientRect().top}scrollAnchorAt(e){let n=this.lineBlockAtHeight(e+8);return n.from>=this.viewport.from||this.viewportLines[0].top-e>200?n:this.viewportLines[0]}elementAtHeight(e){return bs(this.heightMap.blockAt(this.scaler.fromDOM(e),this.heightOracle,0,0),this.scaler)}get docHeight(){return this.scaler.toDOM(this.heightMap.height)}get contentHeight(){return this.docHeight+this.paddingTop+this.paddingBottom}}class il{constructor(e,n){this.from=e,this.to=n}}function uR(t,e,n){let r=[],i=t,s=0;return X.spans(n,t,e,{span(){},point(o,l){o>i&&(r.push({from:i,to:o}),s+=o-i),i=l}},20),i<e&&(r.push({from:i,to:e}),s+=e-i),{total:s,ranges:r}}function sl({total:t,ranges:e},n){if(n<=0)return e[0].from;if(n>=1)return e[e.length-1].to;let r=Math.floor(t*n);for(let i=0;;i++){let{from:s,to:o}=e[i],l=o-s;if(r<=l)return s+r;r-=l}}function ol(t,e){let n=0;for(let{from:r,to:i}of t.ranges){if(e<=i){n+=e-r;break}n+=i-r}return n/t.total}function hR(t,e){for(let n of t)if(e(n))return n}const jm={toDOM(t){return t},fromDOM(t){return t},scale:1,eq(t){return t==this}};function Vm(t){let e=t.facet(Za).filter(r=>typeof r!="function"),n=t.facet(gd).filter(r=>typeof r!="function");return n.length&&e.push(X.join(n)),e}class Sd{constructor(e,n,r){let i=0,s=0,o=0;this.viewports=r.map(({from:l,to:a})=>{let c=n.lineAt(l,he.ByPos,e,0,0).top,u=n.lineAt(a,he.ByPos,e,0,0).bottom;return i+=u-c,{from:l,to:a,top:c,bottom:u,domTop:0,domBottom:0}}),this.scale=(7e6-i)/(n.height-i);for(let l of this.viewports)l.domTop=o+(l.top-s)*this.scale,o=l.domBottom=l.domTop+(l.bottom-l.top),s=l.bottom}toDOM(e){for(let n=0,r=0,i=0;;n++){let s=n<this.viewports.length?this.viewports[n]:null;if(!s||e<s.top)return i+(e-r)*this.scale;if(e<=s.bottom)return s.domTop+(e-s.top);r=s.bottom,i=s.domBottom}}fromDOM(e){for(let n=0,r=0,i=0;;n++){let s=n<this.viewports.length?this.viewports[n]:null;if(!s||e<s.domTop)return r+(e-i)/this.scale;if(e<=s.domBottom)return s.top+(e-s.domTop);r=s.bottom,i=s.domBottom}}eq(e){return e instanceof Sd?this.scale==e.scale&&this.viewports.length==e.viewports.length&&this.viewports.every((n,r)=>n.from==e.viewports[r].from&&n.to==e.viewports[r].to):!1}}function bs(t,e){if(e.scale==1)return t;let n=e.toDOM(t.top),r=e.toDOM(t.bottom);return new $t(t.from,t.length,n,r-n,Array.isArray(t._content)?t._content.map(i=>bs(i,e)):t._content)}const ll=z.define({combine:t=>t.join(" ")}),Vh=z.define({combine:t=>t.indexOf(!0)>-1}),Hh=hr.newName(),yw=hr.newName(),ww=hr.newName(),Sw={"&light":"."+yw,"&dark":"."+ww};function Uh(t,e,n){return new hr(e,{finish(r){return/&/.test(r)?r.replace(/&\w*/,i=>{if(i=="&")return t;if(!n||!n[i])throw new RangeError(`Unsupported selector: ${i}`);return n[i]}):t+" "+r}})}const fR=Uh("."+Hh,{"&":{position:"relative !important",boxSizing:"border-box","&.cm-focused":{outline:"1px dotted #212121"},display:"flex !important",flexDirection:"column"},".cm-scroller":{display:"flex !important",alignItems:"flex-start !important",fontFamily:"monospace",lineHeight:1.4,height:"100%",overflowX:"auto",position:"relative",zIndex:0,overflowAnchor:"none"},".cm-content":{margin:0,flexGrow:2,flexShrink:0,display:"block",whiteSpace:"pre",wordWrap:"normal",boxSizing:"border-box",minHeight:"100%",padding:"4px 0",outline:"none","&[contenteditable=true]":{WebkitUserModify:"read-write-plaintext-only"}},".cm-lineWrapping":{whiteSpace_fallback:"pre-wrap",whiteSpace:"break-spaces",wordBreak:"break-word",overflowWrap:"anywhere",flexShrink:1},"&light .cm-content":{caretColor:"black"},"&dark .cm-content":{caretColor:"white"},".cm-line":{display:"block",padding:"0 2px 0 6px"},".cm-layer":{userSelect:"none",position:"absolute",left:0,top:0,contain:"size style","& > *":{position:"absolute"}},"&light .cm-selectionBackground":{background:"#d9d9d9"},"&dark .cm-selectionBackground":{background:"#222"},"&light.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground":{background:"#d7d4f0"},"&dark.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground":{background:"#233"},".cm-cursorLayer":{pointerEvents:"none"},"&.cm-focused > .cm-scroller > .cm-cursorLayer":{animation:"steps(1) cm-blink 1.2s infinite"},"@keyframes cm-blink":{"0%":{},"50%":{opacity:0},"100%":{}},"@keyframes cm-blink2":{"0%":{},"50%":{opacity:0},"100%":{}},".cm-cursor, .cm-dropCursor":{borderLeft:"1.2px solid black",marginLeft:"-0.6px",pointerEvents:"none"},".cm-cursor":{display:"none"},"&dark .cm-cursor":{borderLeftColor:"#ddd"},".cm-selectionHandle":{backgroundColor:"currentColor",width:"1.5px"},".cm-selectionHandle-start::before, .cm-selectionHandle-end::before":{content:'""',backgroundColor:"inherit",borderRadius:"50%",width:"8px",height:"8px",position:"absolute",left:"-3.25px"},".cm-selectionHandle-start::before":{top:"-8px"},".cm-selectionHandle-end::before":{bottom:"-8px"},".cm-dropCursor":{position:"absolute"},"&.cm-focused > .cm-scroller > .cm-cursorLayer .cm-cursor":{display:"block"},".cm-iso":{unicodeBidi:"isolate"},".cm-announced":{position:"fixed",top:"-10000px"},"@media print":{".cm-announced":{display:"none"}},"&light .cm-activeLine":{backgroundColor:"#cceeff44"},"&dark .cm-activeLine":{backgroundColor:"#99eeff33"},"&light .cm-specialChar":{color:"red"},"&dark .cm-specialChar":{color:"#f78"},".cm-gutters":{flexShrink:0,display:"flex",height:"100%",boxSizing:"border-box",zIndex:200},".cm-gutters-before":{insetInlineStart:0},".cm-gutters-after":{insetInlineEnd:0},"&light .cm-gutters":{backgroundColor:"#f5f5f5",color:"#6c6c6c",border:"0px solid #ddd","&.cm-gutters-before":{borderRightWidth:"1px"},"&.cm-gutters-after":{borderLeftWidth:"1px"}},"&dark .cm-gutters":{backgroundColor:"#333338",color:"#ccc"},".cm-gutter":{display:"flex !important",flexDirection:"column",flexShrink:0,boxSizing:"border-box",minHeight:"100%",overflow:"hidden"},".cm-gutterElement":{boxSizing:"border-box"},".cm-lineNumbers .cm-gutterElement":{padding:"0 3px 0 5px",minWidth:"20px",textAlign:"right",whiteSpace:"nowrap"},"&light .cm-activeLineGutter":{backgroundColor:"#e2f2ff"},"&dark .cm-activeLineGutter":{backgroundColor:"#222227"},".cm-panels":{boxSizing:"border-box",position:"sticky",left:0,right:0,zIndex:300},"&light .cm-panels":{backgroundColor:"#f5f5f5",color:"black"},".cm-panels-top":{top:"0"},".cm-panels-bottom":{bottom:"0"},"&light .cm-panels-top":{borderBottom:"1px solid #ddd"},"&light .cm-panels-bottom":{borderTop:"1px solid #ddd"},"&dark .cm-panels":{backgroundColor:"#333338",color:"white"},".cm-dialog":{padding:"2px 19px 4px 6px",position:"relative","& label":{fontSize:"80%"}},".cm-dialog-close":{position:"absolute",top:"3px",right:"4px",backgroundColor:"inherit",border:"none",font:"inherit",fontSize:"14px",padding:"0"},".cm-tab":{display:"inline-block",overflow:"hidden",verticalAlign:"bottom"},".cm-widgetBuffer":{verticalAlign:"baseline",height:"1em",width:0,display:"inline"},".cm-placeholder":{color:"#888",display:"inline-block",verticalAlign:"top",userSelect:"none"},".cm-highlightSpace":{background:"radial-gradient(circle at 50% 55%, #aaa 20%, transparent 0) no-repeat",backgroundSize:".4em",backgroundPosition:"calc(min(50%, 0px)) center"},".cm-highlightTab":{backgroundImage:`url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="200" height="20"><path stroke="%23888" stroke-width="1" fill="none" d="M1 10H196L190 5M190 15L196 10M197 4L197 16"/></svg>')`,backgroundSize:"auto 100%",backgroundPosition:"right 90%",backgroundRepeat:"no-repeat"},".cm-trailingSpace":{backgroundColor:"#ff332255"},".cm-button":{verticalAlign:"middle",color:"inherit",fontSize:"70%",padding:".2em 1em",borderRadius:"1px"},"&light .cm-button":{backgroundImage:"linear-gradient(#eff1f5, #d9d9df)",border:"1px solid #888","&:active":{backgroundImage:"linear-gradient(#b4b4b4, #d0d3d6)"}},"&dark .cm-button":{backgroundImage:"linear-gradient(#393939, #111)",border:"1px solid #888","&:active":{backgroundImage:"linear-gradient(#111, #333)"}},".cm-textfield":{verticalAlign:"middle",color:"inherit",fontSize:"70%",border:"1px solid silver",padding:".2em .5em"},"&light .cm-textfield":{backgroundColor:"white"},"&dark .cm-textfield":{border:"1px solid #555",backgroundColor:"inherit"}},Sw),dR={childList:!0,characterData:!0,subtree:!0,attributes:!0,characterDataOldValue:!0},eu=_.ie&&_.ie_version<=11;class pR{constructor(e){this.view=e,this.active=!1,this.editContext=null,this.selectionRange=new Vb,this.selectionChanged=!1,this.delayedFlush=-1,this.resizeTimeout=-1,this.queue=[],this.delayedAndroidKey=null,this.flushingAndroidKey=-1,this.lastChange=0,this.scrollTargets=[],this.intersection=null,this.resizeScroll=null,this.intersecting=!1,this.gapIntersection=null,this.gaps=[],this.printQuery=null,this.parentCheck=-1,this.dom=e.contentDOM,this.observer=new MutationObserver(n=>{for(let r of n)this.queue.push(r);(_.ie&&_.ie_version<=11||_.ios&&e.composing)&&n.some(r=>r.type=="childList"&&r.removedNodes.length||r.type=="characterData"&&r.oldValue.length>r.target.nodeValue.length)?this.flushSoon():this.flush()}),window.EditContext&&_.android&&e.constructor.EDIT_CONTEXT!==!1&&!(_.chrome&&_.chrome_version<126)&&(this.editContext=new gR(e),e.state.facet(Rn)&&(e.contentDOM.editContext=this.editContext.editContext)),eu&&(this.onCharData=n=>{this.queue.push({target:n.target,type:"characterData",oldValue:n.prevValue}),this.flushSoon()}),this.onSelectionChange=this.onSelectionChange.bind(this),this.onResize=this.onResize.bind(this),this.onPrint=this.onPrint.bind(this),this.onScroll=this.onScroll.bind(this),window.matchMedia&&(this.printQuery=window.matchMedia("print")),typeof ResizeObserver=="function"&&(this.resizeScroll=new ResizeObserver(()=>{var n;((n=this.view.docView)===null||n===void 0?void 0:n.lastUpdate)<Date.now()-75&&this.onResize()}),this.resizeScroll.observe(e.scrollDOM)),this.addWindowListeners(this.win=e.win),this.start(),typeof IntersectionObserver=="function"&&(this.intersection=new IntersectionObserver(n=>{this.parentCheck<0&&(this.parentCheck=setTimeout(this.listenForScroll.bind(this),1e3)),n.length>0&&n[n.length-1].intersectionRatio>0!=this.intersecting&&(this.intersecting=!this.intersecting,this.intersecting!=this.view.inView&&this.onScrollChanged(document.createEvent("Event")))},{threshold:[0,.001]}),this.intersection.observe(this.dom),this.gapIntersection=new IntersectionObserver(n=>{n.length>0&&n[n.length-1].intersectionRatio>0&&this.onScrollChanged(document.createEvent("Event"))},{})),this.listenForScroll(),this.readSelectionRange()}onScrollChanged(e){this.view.inputState.runHandlers("scroll",e),this.intersecting&&this.view.measure()}onScroll(e){this.intersecting&&this.flush(!1),this.editContext&&this.view.requestMeasure(this.editContext.measureReq),this.onScrollChanged(e)}onResize(){this.resizeTimeout<0&&(this.resizeTimeout=setTimeout(()=>{this.resizeTimeout=-1,this.view.requestMeasure()},50))}onPrint(e){(e.type=="change"||!e.type)&&!e.matches||(this.view.viewState.printing=!0,this.view.measure(),setTimeout(()=>{this.view.viewState.printing=!1,this.view.requestMeasure()},500))}updateGaps(e){if(this.gapIntersection&&(e.length!=this.gaps.length||this.gaps.some((n,r)=>n!=e[r]))){this.gapIntersection.disconnect();for(let n of e)this.gapIntersection.observe(n);this.gaps=e}}onSelectionChange(e){let n=this.selectionChanged;if(!this.readSelectionRange()||this.delayedAndroidKey)return;let{view:r}=this,i=this.selectionRange;if(r.state.facet(Rn)?r.root.activeElement!=this.dom:!_s(this.dom,i))return;let s=i.anchorNode&&r.docView.tile.nearest(i.anchorNode);if(s&&s.isWidget()&&s.widget.ignoreEvent(e)){n||(this.selectionChanged=!1);return}(_.ie&&_.ie_version<=11||_.android&&_.chrome)&&!r.state.selection.main.empty&&i.focusNode&&qs(i.focusNode,i.focusOffset,i.anchorNode,i.anchorOffset)?this.flushSoon():this.flush(!1)}readSelectionRange(){let{view:e}=this,n=uo(e.root);if(!n)return!1;let r=_.safari&&e.root.nodeType==11&&e.root.activeElement==this.dom&&mR(this.view,n)||n;if(!r||this.selectionRange.eq(r))return!1;let i=_s(this.dom,r);return i&&!this.selectionChanged&&e.inputState.lastFocusTime>Date.now()-200&&e.inputState.lastTouchTime<Date.now()-300&&Ub(this.dom,r)?(this.view.inputState.lastFocusTime=0,e.docView.updateSelection(),!1):(this.selectionRange.setRange(r),i&&(this.selectionChanged=!0),!0)}setSelectionRange(e,n){this.selectionRange.set(e.node,e.offset,n.node,n.offset),this.selectionChanged=!1}clearSelectionRange(){this.selectionRange.set(null,0,null,0)}listenForScroll(){this.parentCheck=-1;let e=0,n=null;for(let r=this.dom;r;)if(Fe(r))!n&&e<this.scrollTargets.length&&this.scrollTargets[e]==r?e++:n||(n=this.scrollTargets.slice(0,e)),n&&n.push(r),r=r.assignedSlot||r.parentNode;else if(r.nodeType==11)r=r.host;else break;if(e<this.scrollTargets.length&&!n&&(n=this.scrollTargets.slice(0,e)),n){for(let r of this.scrollTargets)r.removeEventListener("scroll",this.onScroll);for(let r of this.scrollTargets=n)r.addEventListener("scroll",this.onScroll)}}ignore(e){if(!this.active)return e();try{return this.stop(),e()}finally{this.start(),this.clear()}}start(){this.active||(this.observer.observe(this.dom,dR),eu&&this.dom.addEventListener("DOMCharacterDataModified",this.onCharData),this.active=!0)}stop(){this.active&&(this.active=!1,this.observer.disconnect(),eu&&this.dom.removeEventListener("DOMCharacterDataModified",this.onCharData))}clear(){this.processRecords(),this.queue.length=0,this.selectionChanged=!1}delayAndroidKey(e,n){var r;if(!this.delayedAndroidKey){let i=()=>{let s=this.delayedAndroidKey;s&&(this.clearDelayedAndroidKey(),this.view.inputState.lastKeyCode=s.keyCode,this.view.inputState.lastKeyTime=Date.now(),!this.flush()&&s.force&&Pi(this.dom,s.key,s.keyCode))};this.flushingAndroidKey=this.view.win.requestAnimationFrame(i)}(!this.delayedAndroidKey||e=="Enter")&&(this.delayedAndroidKey={key:e,keyCode:n,force:this.lastChange<Date.now()-50||!!(!((r=this.delayedAndroidKey)===null||r===void 0)&&r.force)})}clearDelayedAndroidKey(){this.win.cancelAnimationFrame(this.flushingAndroidKey),this.delayedAndroidKey=null,this.flushingAndroidKey=-1}flushSoon(){this.delayedFlush<0&&(this.delayedFlush=this.view.win.requestAnimationFrame(()=>{this.delayedFlush=-1,this.flush()}))}forceFlush(){this.delayedFlush>=0&&(this.view.win.cancelAnimationFrame(this.delayedFlush),this.delayedFlush=-1),this.flush()}pendingRecords(){for(let e of this.observer.takeRecords())this.queue.push(e);return this.queue}processRecords(){let e=this.pendingRecords();e.length&&(this.queue=[]);let n=-1,r=-1,i=!1;for(let s of e){let o=this.readMutation(s);o&&(o.typeOver&&(i=!0),n==-1?{from:n,to:r}=o:(n=Math.min(o.from,n),r=Math.max(o.to,r)))}return{from:n,to:r,typeOver:i}}readChange(){let{from:e,to:n,typeOver:r}=this.processRecords(),i=this.selectionChanged&&_s(this.dom,this.selectionRange);if(e<0&&!i)return null;e>-1&&(this.lastChange=Date.now()),this.view.inputState.lastFocusTime=0,this.selectionChanged=!1;let s=new LT(this.view,e,n,r);return this.view.docView.domChanged={newSel:s.newSel?s.newSel.main:null},s}flush(e=!0){if(this.delayedFlush>=0||this.delayedAndroidKey)return!1;e&&this.readSelectionRange();let n=this.selectionChanged,r=this.readChange();if(!r)return this.view.requestMeasure(),!1;let i=this.view.state,s=lw(this.view,r);return this.view.state==i&&(r.domChanged||n)&&this.view.update([]),s}readMutation(e){let n=this.view.docView.tile.nearest(e.target);if(!n||n.isWidget())return null;if(n.markDirty(e.type=="attributes"),e.type=="childList"){let r=Hm(n,e.previousSibling||e.target.previousSibling,-1),i=Hm(n,e.nextSibling||e.target.nextSibling,1);return{from:r?n.posAfter(r):n.posAtStart,to:i?n.posBefore(i):n.posAtEnd,typeOver:!1}}else return e.type=="characterData"?{from:n.posAtStart,to:n.posAtEnd,typeOver:e.target.nodeValue==e.oldValue}:null}setWindow(e){e!=this.win&&(this.removeWindowListeners(this.win),this.win=e,this.addWindowListeners(this.win))}addWindowListeners(e){e.addEventListener("resize",this.onResize),this.printQuery?this.printQuery.addEventListener?this.printQuery.addEventListener("change",this.onPrint):this.printQuery.addListener(this.onPrint):e.addEventListener("beforeprint",this.onPrint),e.addEventListener("scroll",this.onScroll),e.document.addEventListener("selectionchange",this.onSelectionChange)}removeWindowListeners(e){e.removeEventListener("scroll",this.onScroll),e.removeEventListener("resize",this.onResize),this.printQuery?this.printQuery.removeEventListener?this.printQuery.removeEventListener("change",this.onPrint):this.printQuery.removeListener(this.onPrint):e.removeEventListener("beforeprint",this.onPrint),e.document.removeEventListener("selectionchange",this.onSelectionChange)}update(e){this.editContext&&(this.editContext.update(e),e.startState.facet(Rn)!=e.state.facet(Rn)&&(e.view.contentDOM.editContext=e.state.facet(Rn)?this.editContext.editContext:null))}destroy(){var e,n,r;this.stop(),(e=this.intersection)===null||e===void 0||e.disconnect(),(n=this.gapIntersection)===null||n===void 0||n.disconnect(),(r=this.resizeScroll)===null||r===void 0||r.disconnect();for(let i of this.scrollTargets)i.removeEventListener("scroll",this.onScroll);this.removeWindowListeners(this.win),clearTimeout(this.parentCheck),clearTimeout(this.resizeTimeout),this.win.cancelAnimationFrame(this.delayedFlush),this.win.cancelAnimationFrame(this.flushingAndroidKey),this.editContext&&(this.view.contentDOM.editContext=null,this.editContext.destroy())}}function Hm(t,e,n){for(;e;){let r=Re.get(e);if(r&&r.parent==t)return r;let i=e.parentNode;e=i!=t.dom?i:n>0?e.nextSibling:e.previousSibling}return null}function Um(t,e){let n=e.startContainer,r=e.startOffset,i=e.endContainer,s=e.endOffset,o=t.docView.domAtPos(t.state.selection.main.anchor,1);return qs(o.node,o.offset,i,s)&&([n,r,i,s]=[i,s,n,r]),{anchorNode:n,anchorOffset:r,focusNode:i,focusOffset:s}}function mR(t,e){if(e.getComposedRanges){let i=e.getComposedRanges(t.root)[0];if(i)return Um(t,i)}let n=null;function r(i){i.preventDefault(),i.stopImmediatePropagation(),n=i.getTargetRanges()[0]}return t.contentDOM.addEventListener("beforeinput",r,!0),t.dom.ownerDocument.execCommand("indent"),t.contentDOM.removeEventListener("beforeinput",r,!0),n?Um(t,n):null}class gR{constructor(e){this.from=0,this.to=0,this.pendingContextChange=null,this.handlers=Object.create(null),this.composing=null,this.resetRange(e.state);let n=this.editContext=new window.EditContext({text:e.state.doc.sliceString(this.from,this.to),selectionStart:this.toContextPos(Math.max(this.from,Math.min(this.to,e.state.selection.main.anchor))),selectionEnd:this.toContextPos(e.state.selection.main.head)});this.handlers.textupdate=r=>{let i=e.state.selection.main,{anchor:s,head:o}=i,l=this.toEditorPos(r.updateRangeStart),a=this.toEditorPos(r.updateRangeEnd);e.inputState.composing>=0&&!this.composing&&(this.composing={contextBase:r.updateRangeStart,editorBase:l,drifted:!1});let c=a-l>r.text.length;l==this.from&&s<this.from?l=s:a==this.to&&s>this.to&&(a=s);let u=aw(e.state.sliceDoc(l,a),r.text,(c?i.from:i.to)-l,c?"end":null);if(!u){let f=R.single(this.toEditorPos(r.selectionStart),this.toEditorPos(r.selectionEnd));zh(f,i)||e.dispatch({selection:f,userEvent:"select"});return}let h={from:u.from+l,to:u.toA+l,insert:te.of(r.text.slice(u.from,u.toB).split(`
`))};if((_.mac||_.android)&&h.from==o-1&&/^\. ?$/.test(r.text)&&e.contentDOM.getAttribute("autocorrect")=="off"&&(h={from:l,to:a,insert:te.of([r.text.replace("."," ")])}),this.pendingContextChange=h,!e.state.readOnly){let f=this.to-this.from+(h.to-h.from+h.insert.length);yd(e,h,R.single(this.toEditorPos(r.selectionStart,f),this.toEditorPos(r.selectionEnd,f)))}this.pendingContextChange&&(this.revertPending(e.state),this.setSelection(e.state)),h.from<h.to&&!h.insert.length&&e.inputState.composing>=0&&!/[\\p{Alphabetic}\\p{Number}_]/.test(n.text.slice(Math.max(0,r.updateRangeStart-1),Math.min(n.text.length,r.updateRangeStart+1)))&&this.handlers.compositionend(r)},this.handlers.characterboundsupdate=r=>{let i=[],s=null;for(let o=this.toEditorPos(r.rangeStart),l=this.toEditorPos(r.rangeEnd);o<l;o++){let a=e.coordsForChar(o);s=a&&new DOMRect(a.left,a.top,a.right-a.left,a.bottom-a.top)||s||new DOMRect,i.push(s)}n.updateCharacterBounds(r.rangeStart,i)},this.handlers.textformatupdate=r=>{let i=[];for(let s of r.getTextFormats()){let o=s.underlineStyle,l=s.underlineThickness;if(!/none/i.test(o)&&!/none/i.test(l)){let a=this.toEditorPos(s.rangeStart),c=this.toEditorPos(s.rangeEnd);if(a<c){let u=`text-decoration: underline ${/^[a-z]/.test(o)?o+" ":o=="Dashed"?"dashed ":o=="Squiggle"?"wavy ":""}${/thin/i.test(l)?1:2}px`;i.push(U.mark({attributes:{style:u}}).range(a,c))}}}e.dispatch({effects:Jy.of(U.set(i))})},this.handlers.compositionstart=()=>{e.inputState.composing<0&&(e.inputState.composing=0,e.inputState.compositionFirstChange=!0)},this.handlers.compositionend=()=>{if(e.inputState.composing=-1,e.inputState.compositionFirstChange=null,this.composing){let{drifted:r}=this.composing;this.composing=null,r&&this.reset(e.state)}};for(let r in this.handlers)n.addEventListener(r,this.handlers[r]);this.measureReq={read:r=>{let i=uo(r.root);i&&i.rangeCount&&this.editContext.updateSelectionBounds(i.getRangeAt(0).getBoundingClientRect())}}}applyEdits(e){let n=0,r=!1,i=this.pendingContextChange;return e.changes.iterChanges((s,o,l,a,c)=>{if(r)return;let u=c.length-(o-s);if(i&&o>=i.to)if(i.from==s&&i.to==o&&i.insert.eq(c)){i=this.pendingContextChange=null,n+=u,this.to+=u;return}else i=null,this.revertPending(e.state);if(s+=n,o+=n,o<=this.from)this.from+=u,this.to+=u;else if(s<this.to){if(s<this.from||o>this.to||this.to-this.from+c.length>3e4){r=!0;return}this.editContext.updateText(this.toContextPos(s),this.toContextPos(o),c.toString()),this.to+=u}n+=u}),i&&!r&&this.revertPending(e.state),!r}update(e){let n=this.pendingContextChange,r=e.startState.selection.main;this.composing&&(this.composing.drifted||!e.changes.touchesRange(r.from,r.to)&&e.transactions.some(i=>!i.isUserEvent("input.type")&&i.changes.touchesRange(this.from,this.to)))?(this.composing.drifted=!0,this.composing.editorBase=e.changes.mapPos(this.composing.editorBase)):!this.applyEdits(e)||!this.rangeIsValid(e.state)?(this.pendingContextChange=null,this.reset(e.state)):(e.docChanged||e.selectionSet||n)&&this.setSelection(e.state),(e.geometryChanged||e.docChanged||e.selectionSet)&&e.view.requestMeasure(this.measureReq)}resetRange(e){let{head:n}=e.selection.main;this.from=Math.max(0,n-1e4),this.to=Math.min(e.doc.length,n+1e4)}reset(e){this.resetRange(e),this.editContext.updateText(0,this.editContext.text.length,e.doc.sliceString(this.from,this.to)),this.setSelection(e)}revertPending(e){let n=this.pendingContextChange;this.pendingContextChange=null,this.editContext.updateText(this.toContextPos(n.from),this.toContextPos(n.from+n.insert.length),e.doc.sliceString(n.from,n.to))}setSelection(e){let{main:n}=e.selection,r=this.toContextPos(Math.max(this.from,Math.min(this.to,n.anchor))),i=this.toContextPos(n.head);(this.editContext.selectionStart!=r||this.editContext.selectionEnd!=i)&&this.editContext.updateSelection(r,i)}rangeIsValid(e){let{head:n}=e.selection.main;return!(this.from>0&&n-this.from<500||this.to<e.doc.length&&this.to-n<500||this.to-this.from>1e4*3)}toEditorPos(e,n=this.to-this.from){e=Math.min(e,n);let r=this.composing;return r&&r.drifted?r.editorBase+(e-r.contextBase):e+this.from}toContextPos(e){let n=this.composing;return n&&n.drifted?n.contextBase+(e-n.editorBase):e-this.from}destroy(){for(let e in this.handlers)this.editContext.removeEventListener(e,this.handlers[e])}}class F{get state(){return this.viewState.state}get viewport(){return this.viewState.viewport}get visibleRanges(){return this.viewState.visibleRanges}get inView(){return this.viewState.inView}get composing(){return!!this.inputState&&this.inputState.composing>0}get compositionStarted(){return!!this.inputState&&this.inputState.composing>=0}get root(){return this._root}get win(){return this.dom.ownerDocument.defaultView||window}constructor(e={}){var n;this.plugins=[],this.pluginMap=new Map,this.editorAttrs={},this.contentAttrs={},this.bidiCache=[],this.destroyed=!1,this.updateState=2,this.measureScheduled=-1,this.measureRequests=[],this.clearAnnouncement=-1,this.contentDOM=document.createElement("div"),this.scrollDOM=document.createElement("div"),this.scrollDOM.tabIndex=-1,this.scrollDOM.className="cm-scroller",this.scrollDOM.appendChild(this.contentDOM),this.announceDOM=document.createElement("div"),this.announceDOM.className="cm-announced",this.announceDOM.setAttribute("aria-live","polite"),this.dom=document.createElement("div"),this.dom.appendChild(this.announceDOM),this.dom.appendChild(this.scrollDOM),e.parent&&e.parent.appendChild(this.dom);let{dispatch:r}=e;this.dispatchTransactions=e.dispatchTransactions||r&&(i=>i.forEach(s=>r(s,this)))||(i=>this.update(i)),this.dispatch=this.dispatch.bind(this),this._root=e.root||Hb(e.parent)||document,this.viewState=new zm(this,e.state||ee.create(e)),e.scrollTo&&e.scrollTo.is(tl)&&(this.viewState.scrollTarget=e.scrollTo.value.clip(this.viewState.state)),this.plugins=this.state.facet(yi).map(i=>new Kc(i));for(let i of this.plugins)i.update(this);this.observer=new pR(this),this.inputState=new BT(this),this.inputState.ensureHandlers(this.plugins),this.docView=new Im(this),this.mountStyles(),this.updateAttrs(),this.updateState=0,this.requestMeasure(),!((n=document.fonts)===null||n===void 0)&&n.ready&&document.fonts.ready.then(()=>{this.viewState.mustMeasureContent="refresh",this.requestMeasure()})}dispatch(...e){let n=e.length==1&&e[0]instanceof Ne?e:e.length==1&&Array.isArray(e[0])?e[0]:[this.state.update(...e)];this.dispatchTransactions(n,this)}update(e){if(this.updateState!=0)throw new Error("Calls to EditorView.update are not allowed while an update is in progress");let n=!1,r=!1,i,s=this.state;for(let f of e){if(f.startState!=s)throw new RangeError("Trying to update state with a transaction that doesn't start from the previous state.");s=f.state}if(this.destroyed){this.viewState.state=s;return}let o=this.hasFocus,l=0,a=null;e.some(f=>f.annotation(pw))?(this.inputState.notifiedFocused=o,l=1):o!=this.inputState.notifiedFocused&&(this.inputState.notifiedFocused=o,a=mw(s,o),a||(l=1));let c=this.observer.delayedAndroidKey,u=null;if(c?(this.observer.clearDelayedAndroidKey(),u=this.observer.readChange(),(u&&!this.state.doc.eq(s.doc)||!this.state.selection.eq(s.selection))&&(u=null)):this.observer.clear(),s.facet(ee.phrases)!=this.state.facet(ee.phrases))return this.setState(s);i=ya.create(this,s,e),i.flags|=l;let h=this.viewState.scrollTarget;try{this.updateState=2;for(let f of e){if(h&&(h=h.map(f.changes)),f.scrollIntoView){let{main:d}=f.state.selection,{x:p,y:m}=this.state.facet(F.cursorScrollMargin);h=new Ei(d.empty?d:R.cursor(d.head,d.head>d.anchor?-1:1),"nearest","nearest",m,p)}for(let d of f.effects)d.is(tl)&&(h=d.value.clip(this.state))}this.viewState.update(i,h),this.bidiCache=xa.update(this.bidiCache,i.changes),i.empty||(this.updatePlugins(i),this.inputState.update(i)),n=this.docView.update(i),this.state.facet(Cs)!=this.styleModules&&this.mountStyles(),r=this.updateAttrs(),this.showAnnouncements(e),this.docView.updateSelection(n,e.some(f=>f.isUserEvent("select.pointer")))}finally{this.updateState=0}if(i.startState.facet(ll)!=i.state.facet(ll)&&(this.viewState.mustMeasureContent=!0),(n||r||h||this.viewState.mustEnforceCursorAssoc||this.viewState.mustMeasureContent)&&this.requestMeasure(),n&&this.docViewUpdate(),!i.empty)for(let f of this.state.facet(_h))try{f(i)}catch(d){st(this.state,d,"update listener")}(a||u)&&Promise.resolve().then(()=>{a&&this.state==a.startState&&this.dispatch(a),u&&!lw(this,u)&&c.force&&Pi(this.contentDOM,c.key,c.keyCode)})}setState(e){if(this.updateState!=0)throw new Error("Calls to EditorView.setState are not allowed while an update is in progress");if(this.destroyed){this.viewState.state=e;return}this.updateState=2;let n=this.hasFocus;try{for(let r of this.plugins)r.destroy(this);this.viewState=new zm(this,e),this.plugins=e.facet(yi).map(r=>new Kc(r)),this.pluginMap.clear();for(let r of this.plugins)r.update(this);this.docView.destroy(),this.docView=new Im(this),this.inputState.ensureHandlers(this.plugins),this.mountStyles(),this.updateAttrs(),this.bidiCache=[]}finally{this.updateState=0}n&&this.focus(),this.requestMeasure()}updatePlugins(e){let n=e.startState.facet(yi),r=e.state.facet(yi);if(n!=r){let i=[];for(let s of r){let o=n.indexOf(s);if(o<0)i.push(new Kc(s));else{let l=this.plugins[o];l.mustUpdate=e,i.push(l)}}for(let s of this.plugins)s.mustUpdate!=e&&s.destroy(this);this.plugins=i,this.pluginMap.clear()}else for(let i of this.plugins)i.mustUpdate=e;for(let i=0;i<this.plugins.length;i++)this.plugins[i].update(this);n!=r&&this.inputState.ensureHandlers(this.plugins)}docViewUpdate(){for(let e of this.plugins){let n=e.value;if(n&&n.docViewUpdate)try{n.docViewUpdate(this)}catch(r){st(this.state,r,"doc view update listener")}}}measure(e=!0){if(this.destroyed)return;if(this.measureScheduled>-1&&this.win.cancelAnimationFrame(this.measureScheduled),this.observer.delayedAndroidKey){this.measureScheduled=-1,this.requestMeasure();return}this.measureScheduled=0,e&&this.observer.forceFlush();let n=null,r=this.viewState.scrollParent,i=this.viewState.getScrollOffset(),{scrollAnchorPos:s,scrollAnchorHeight:o,scaleY:l}=this.viewState;Math.abs(i-this.viewState.scrollOffset)>1&&(o=-1),this.viewState.scrollAnchorHeight=-1;try{for(let a=0;;a++){if(o<0){if(Ny(r||this.win))s=-1,o=this.viewState.heightMap.height/this.viewState.scaleY;else{let p=this.viewState.scrollAnchorAt(i);s=p.from,o=p.top}l=this.viewState.scaleY}this.updateState=1;let c=this.viewState.measure();if(!c&&!this.measureRequests.length&&this.viewState.scrollTarget==null)break;if(a>5){console.warn(this.measureRequests.length?"Measure loop restarted more than 5 times":"Viewport failed to stabilize");break}let u=[];c&4||([this.measureRequests,u]=[u,this.measureRequests]);let h=u.map(p=>{try{return p.read(this)}catch(m){return st(this.state,m),$m}}),f=ya.create(this,this.state,[]),d=!1;f.flags|=c,n?n.flags|=c:n=f,this.updateState=2,f.empty||(this.updatePlugins(f),this.inputState.update(f),this.updateAttrs(),d=this.docView.update(f),d&&this.docViewUpdate());for(let p=0;p<u.length;p++)if(h[p]!=$m)try{let m=u[p];m.write&&m.write(h[p],this)}catch(m){st(this.state,m)}if(d&&this.docView.updateSelection(!0),!f.viewportChanged&&this.measureRequests.length==0){if(this.viewState.editorHeight)if(this.viewState.scrollTarget){this.docView.scrollIntoView(this.viewState.scrollTarget),this.viewState.scrollTarget=null,o=-1;continue}else{let m=(s<0?this.viewState.heightMap.height:this.viewState.lineBlockAt(s).top)/this.viewState.scaleY-o/l;if((m>1||m<-1)&&!(_.ios&&this.inputState.lastIOSMomentumScroll>Date.now()-100)&&(r==this.scrollDOM||this.hasFocus||Math.max(this.inputState.lastWheelEvent,this.inputState.lastTouchTime)>Date.now()-100)){i=i+m,r?s<0?r.scrollTop=r.scrollHeight:r.scrollTop+=m:this.win.scrollBy(0,m),o=-1;continue}}break}}}finally{this.updateState=0,this.measureScheduled=-1}if(n&&!n.empty)for(let a of this.state.facet(_h))a(n)}get themeClasses(){return Hh+" "+(this.state.facet(Vh)?ww:yw)+" "+this.state.facet(ll)}updateAttrs(){let e=Gm(this,Zy,{class:"cm-editor"+(this.hasFocus?" cm-focused ":" ")+this.themeClasses}),n={spellcheck:"false",autocorrect:"off",autocapitalize:"off",writingsuggestions:"false",translate:"no",contenteditable:this.state.facet(Rn)?"true":"false",class:"cm-content",style:`${_.tabSize}: ${this.state.tabSize}`,role:"textbox","aria-multiline":"true"};this.state.readOnly&&(n["aria-readonly"]="true"),Gm(this,md,n);let r=this.observer.ignore(()=>{let i=xm(this.contentDOM,this.contentAttrs,n),s=xm(this.dom,this.editorAttrs,e);return i||s});return this.editorAttrs=e,this.contentAttrs=n,r}showAnnouncements(e){let n=!0;for(let r of e)for(let i of r.effects)if(i.is(F.announce)){n&&(this.announceDOM.textContent="",this.win.clearTimeout(this.clearAnnouncement),this.clearAnnouncement=this.win.setTimeout(()=>{this.announceDOM.textContent=" "},200),n=!1);let s=this.announceDOM.appendChild(document.createElement("div"));s.textContent=i.value}}mountStyles(){this.styleModules=this.state.facet(Cs);let e=this.state.facet(F.cspNonce);hr.mount(this.root,this.styleModules.concat(fR).reverse(),e?{nonce:e}:void 0)}readMeasured(){if(this.updateState==2)throw new Error("Reading the editor layout isn't allowed during an update");this.updateState==0&&this.measureScheduled>-1&&this.measure(!1)}requestMeasure(e){if(this.measureScheduled<0&&(this.measureScheduled=this.win.requestAnimationFrame(()=>this.measure())),e){if(this.measureRequests.indexOf(e)>-1)return;if(e.key!=null){for(let n=0;n<this.measureRequests.length;n++)if(this.measureRequests[n].key===e.key){this.measureRequests[n]=e;return}}this.measureRequests.push(e)}}plugin(e){let n=this.pluginMap.get(e);return(n===void 0||n&&n.plugin!=e)&&this.pluginMap.set(e,n=this.plugins.find(r=>r.plugin==e)||null),n&&n.update(this).value}get documentTop(){return this.contentDOM.getBoundingClientRect().top+this.viewState.paddingTop}get documentPadding(){return{top:this.viewState.paddingTop,bottom:this.viewState.paddingBottom}}get scaleX(){return this.viewState.scaleX}get scaleY(){return this.viewState.scaleY}elementAtHeight(e){return this.readMeasured(),this.viewState.elementAtHeight(e)}lineBlockAtHeight(e){return this.readMeasured(),this.viewState.lineBlockAtHeight(e)}get viewportLineBlocks(){return this.viewState.viewportLines}lineBlockAt(e){return this.viewState.lineBlockAt(e)}get contentHeight(){return this.viewState.contentHeight}moveByChar(e,n,r){return Jc(this,e,Am(this,e,n,r))}moveByGroup(e,n){return Jc(this,e,Am(this,e,n,r=>RT(this,e.head,r)))}visualLineSide(e,n){return n?R.cursor(e.to,1):R.cursor(e.from,-1)}moveToLineBoundary(e,n,r=!0){return TT(this,e,n,r)}moveVertically(e,n,r){return Jc(this,e,MT(this,e,n,r))}domAtPos(e,n=1){return this.docView.domAtPos(e,n)}posAtDOM(e,n=0){return this.docView.posFromDOM(e,n)}posAtCoords(e,n=!0){this.readMeasured();let r=Fh(this,e,n);return r&&r.pos}posAndSideAtCoords(e,n=!0){return this.readMeasured(),Fh(this,e,n)}coordsAtPos(e,n=1){this.readMeasured();let r=this.state.doc.lineAt(e),i=this.bidiSpans(r),s=i[pn.find(i,e-r.from,-1,n)];return r.length&&(e==r.from&&n<0||e==r.to&&n>0)&&s.dir!=this.textDirectionAt(r.from)&&(e==r.to?(e=r.from+s.from,n=1):(e=r.from+s.to,n=-1)),this.docView.coordsAt(e,n,s.dir==ue.RTL)}coordsForChar(e){return this.readMeasured(),this.docView.coordsForChar(e)}get defaultCharacterWidth(){return this.viewState.heightOracle.charWidth}get defaultLineHeight(){return this.viewState.heightOracle.lineHeight}get textDirection(){return this.viewState.defaultTextDirection}textDirectionAt(e){return!this.state.facet(Qy)||e<this.viewport.from||e>this.viewport.to?this.textDirection:(this.readMeasured(),this.docView.textDirectionAt(e))}get lineWrapping(){return this.viewState.heightOracle.lineWrapping}bidiSpans(e){if(e.length>vR)return zy(e.length);let n=this.textDirectionAt(e.from),r;for(let s of this.bidiCache)if(s.from==e.from&&s.dir==n&&(s.fresh||Fy(s.isolates,r=Tm(this,e))))return s.order;r||(r=Tm(this,e));let i=Jb(e.text,n,r);return this.bidiCache.push(new xa(e.from,e.to,n,r,!0,i)),i}get hasFocus(){var e;return(this.dom.ownerDocument.hasFocus()||_.safari&&((e=this.inputState)===null||e===void 0?void 0:e.lastContextMenu)>Date.now()-3e4)&&this.root.activeElement==this.contentDOM}focus(){this.observer.ignore(()=>{Oy(this.contentDOM),this.docView.updateSelection()})}setRoot(e){this._root!=e&&(this._root=e,this.observer.setWindow((e.nodeType==9?e:e.ownerDocument).defaultView||window),this.mountStyles())}destroy(){this.root.activeElement==this.contentDOM&&this.contentDOM.blur();for(let e of this.plugins)e.destroy(this);this.plugins=[],this.inputState.destroy(),this.docView.destroy(),this.dom.remove(),this.observer.destroy(),this.win.clearTimeout(this.clearAnnouncement),this.measureScheduled>-1&&this.win.cancelAnimationFrame(this.measureScheduled),this.destroyed=!0}static scrollIntoView(e,n={}){var r,i,s,o;return tl.of(new Ei(typeof e=="number"?R.cursor(e):e,(r=n.y)!==null&&r!==void 0?r:"nearest",(i=n.x)!==null&&i!==void 0?i:"nearest",(s=n.yMargin)!==null&&s!==void 0?s:5,(o=n.xMargin)!==null&&o!==void 0?o:5))}scrollSnapshot(){let{scrollTop:e,scrollLeft:n}=this.scrollDOM,r=this.viewState.scrollAnchorAt(e);return tl.of(new Ei(R.cursor(r.from),"start","start",r.top-e,n,!0))}setTabFocusMode(e){e==null?this.inputState.tabFocusMode=this.inputState.tabFocusMode<0?0:-1:typeof e=="boolean"?this.inputState.tabFocusMode=e?0:-1:this.inputState.tabFocusMode!=0&&(this.inputState.tabFocusMode=Date.now()+e)}static domEventHandlers(e){return Me.define(()=>({}),{eventHandlers:e})}static domEventObservers(e){return Me.define(()=>({}),{eventObservers:e})}static theme(e,n){let r=hr.newName(),i=[ll.of(r),Cs.of(Uh(`.${r}`,e))];return n&&n.dark&&i.push(Vh.of(!0)),i}static baseTheme(e){return Zr.lowest(Cs.of(Uh("."+Hh,e,Sw)))}static findFromDOM(e){var n;let r=e.querySelector(".cm-content"),i=r&&Re.get(r)||Re.get(e);return((n=i==null?void 0:i.root)===null||n===void 0?void 0:n.view)||null}}F.styleModule=Cs;F.inputHandler=Gy;F.clipboardInputFilter=dd;F.clipboardOutputFilter=pd;F.scrollHandler=Xy;F.focusChangeEffect=Ky;F.perLineTextDirection=Qy;F.exceptionSink=$y;F.updateListener=_h;F.editable=Rn;F.mouseSelectionStyle=Uy;F.dragMovesSelection=Hy;F.clickAddsSelectionRange=Vy;F.decorations=Za;F.blockWrappers=ew;F.outerDecorations=gd;F.atomicRanges=Ao;F.bidiIsolatedRanges=tw;F.cursorScrollMargin=z.define({combine:t=>{let e=5,n=5;for(let r of t)typeof r=="number"?e=n=r:{x:e,y:n}=r;return{x:e,y:n}}});F.scrollMargins=nw;F.darkTheme=Vh;F.cspNonce=z.define({combine:t=>t.length?t[0]:""});F.contentAttributes=md;F.editorAttributes=Zy;F.lineWrapping=F.contentAttributes.of({class:"cm-lineWrapping"});F.announce=K.define();const vR=4096,$m={};class xa{constructor(e,n,r,i,s,o){this.from=e,this.to=n,this.dir=r,this.isolates=i,this.fresh=s,this.order=o}static update(e,n){if(n.empty&&!e.some(s=>s.fresh))return e;let r=[],i=e.length?e[e.length-1].dir:ue.LTR;for(let s=Math.max(0,e.length-10);s<e.length;s++){let o=e[s];o.dir==i&&!n.touchesRange(o.from,o.to)&&r.push(new xa(n.mapPos(o.from,1),n.mapPos(o.to,-1),o.dir,o.isolates,!1,o.order))}return r}}function Gm(t,e,n){for(let r=t.state.facet(e),i=r.length-1;i>=0;i--){let s=r[i],o=typeof s=="function"?s(t):s;o&&ud(o,n)}return n}const yR=_.mac?"mac":_.windows?"win":_.linux?"linux":"key";function wR(t,e){const n=t.split(/-(?!$)/);let r=n[n.length-1];r=="Space"&&(r=" ");let i,s,o,l;for(let a=0;a<n.length-1;++a){const c=n[a];if(/^(cmd|meta|m)$/i.test(c))l=!0;else if(/^a(lt)?$/i.test(c))i=!0;else if(/^(c|ctrl|control)$/i.test(c))s=!0;else if(/^s(hift)?$/i.test(c))o=!0;else if(/^mod$/i.test(c))e=="mac"?l=!0:s=!0;else throw new Error("Unrecognized modifier name: "+c)}return i&&(r="Alt-"+r),s&&(r="Ctrl-"+r),l&&(r="Meta-"+r),o&&(r="Shift-"+r),r}function al(t,e,n){return e.altKey&&(t="Alt-"+t),e.ctrlKey&&(t="Ctrl-"+t),e.metaKey&&(t="Meta-"+t),n!==!1&&e.shiftKey&&(t="Shift-"+t),t}const SR=Zr.default(F.domEventHandlers({keydown(t,e){return xw(kw(e.state),t,e,"editor")}})),rc=z.define({enables:SR}),Km=new WeakMap;function kw(t){let e=t.facet(rc),n=Km.get(e);return n||Km.set(e,n=CR(e.reduce((r,i)=>r.concat(i),[]))),n}function kR(t,e,n){return xw(kw(t.state),e,t,n)}let Gn=null;const xR=4e3;function CR(t,e=yR){let n=Object.create(null),r=Object.create(null),i=(o,l)=>{let a=r[o];if(a==null)r[o]=l;else if(a!=l)throw new Error("Key binding "+o+" is used both as a regular binding and as a multi-stroke prefix")},s=(o,l,a,c,u)=>{var h,f;let d=n[o]||(n[o]=Object.create(null)),p=l.split(/ (?!$)/).map(g=>wR(g,e));for(let g=1;g<p.length;g++){let v=p.slice(0,g).join(" ");i(v,!0),d[v]||(d[v]={preventDefault:!0,stopPropagation:!1,run:[y=>{let x=Gn={view:y,prefix:v,scope:o};return setTimeout(()=>{Gn==x&&(Gn=null)},xR),!0}]})}let m=p.join(" ");i(m,!1);let w=d[m]||(d[m]={preventDefault:!1,stopPropagation:!1,run:((f=(h=d._any)===null||h===void 0?void 0:h.run)===null||f===void 0?void 0:f.slice())||[]});a&&w.run.push(a),c&&(w.preventDefault=!0),u&&(w.stopPropagation=!0)};for(let o of t){let l=o.scope?o.scope.split(" "):["editor"];if(o.any)for(let c of l){let u=n[c]||(n[c]=Object.create(null));u._any||(u._any={preventDefault:!1,stopPropagation:!1,run:[]});let{any:h}=o;for(let f in u)u[f].run.push(d=>h(d,$h))}let a=o[e]||o.key;if(a)for(let c of l)s(c,a,o.run,o.preventDefault,o.stopPropagation),o.shift&&s(c,"Shift-"+a,o.shift,o.preventDefault,o.stopPropagation)}return n}let $h=null;function xw(t,e,n,r){$h=e;let i=_b(e),s=ft(i,0),o=fn(s)==i.length&&i!=" ",l="",a=!1,c=!1,u=!1;Gn&&Gn.view==n&&Gn.scope==r&&(l=Gn.prefix+" ",uw.indexOf(e.keyCode)<0&&(c=!0,Gn=null));let h=new Set,f=w=>{if(w){for(let g of w.run)if(!h.has(g)&&(h.add(g),g(n)))return w.stopPropagation&&(u=!0),!0;w.preventDefault&&(w.stopPropagation&&(u=!0),c=!0)}return!1},d=t[r],p,m;return d&&(f(d[l+al(i,e,!o)])?a=!0:o&&(e.altKey||e.metaKey||e.ctrlKey)&&!(_.windows&&e.ctrlKey&&e.altKey)&&!(_.mac&&e.altKey&&!(e.ctrlKey||e.metaKey))&&(p=fr[e.keyCode])&&p!=i?(f(d[l+al(p,e,!0)])||e.shiftKey&&(m=ao[e.keyCode])!=i&&m!=p&&f(d[l+al(m,e,!1)]))&&(a=!0):o&&e.shiftKey&&f(d[l+al(i,e,!0)])&&(a=!0),!a&&f(d._any)&&(a=!0)),c&&(a=!0),a&&u&&e.stopPropagation(),$h=null,a}class _r{constructor(e,n,r,i,s){this.className=e,this.left=n,this.top=r,this.width=i,this.height=s}draw(){let e=document.createElement("div");return e.className=this.className,this.adjust(e),e}update(e,n){return n.className!=this.className?!1:(this.adjust(e),!0)}adjust(e){e.style.left=this.left+"px",e.style.top=this.top+"px",this.width!=null&&(e.style.width=this.width+"px"),e.style.height=this.height+"px"}eq(e){return this.left==e.left&&this.top==e.top&&this.width==e.width&&this.height==e.height&&this.className==e.className}static forRange(e,n,r){if(r.empty){let i=e.coordsAtPos(r.head,r.assoc||1);if(!i)return[];let s=Cw(e);return[new _r(n,i.left-s.left,i.top-s.top,null,i.bottom-i.top)]}else return bR(e,n,r)}}function Cw(t){let e=t.scrollDOM.getBoundingClientRect();return{left:(t.textDirection==ue.LTR?e.left:e.right-t.scrollDOM.clientWidth*t.scaleX)-t.scrollDOM.scrollLeft*t.scaleX,top:e.top-t.scrollDOM.scrollTop*t.scaleY}}function Qm(t,e,n,r){let i=t.coordsAtPos(e,n*2);if(!i)return r;let s=t.dom.getBoundingClientRect(),o=(i.top+i.bottom)/2,l=t.posAtCoords({x:s.left+1,y:o}),a=t.posAtCoords({x:s.right-1,y:o});return l==null||a==null?r:{from:Math.max(r.from,Math.min(l,a)),to:Math.min(r.to,Math.max(l,a))}}function bR(t,e,n){if(n.to<=t.viewport.from||n.from>=t.viewport.to)return[];let r=Math.max(n.from,t.viewport.from),i=Math.min(n.to,t.viewport.to),s=t.textDirection==ue.LTR,o=t.contentDOM,l=o.getBoundingClientRect(),a=Cw(t),c=o.querySelector(".cm-line"),u=c&&window.getComputedStyle(c),h=l.left+(u?parseInt(u.paddingLeft)+Math.min(0,parseInt(u.textIndent)):0),f=l.right-(u?parseInt(u.paddingRight):0),d=Wh(t,r,1),p=Wh(t,i,-1),m=d.type==Ge.Text?d:null,w=p.type==Ge.Text?p:null;if(m&&(t.lineWrapping||d.widgetLineBreaks)&&(m=Qm(t,r,1,m)),w&&(t.lineWrapping||p.widgetLineBreaks)&&(w=Qm(t,i,-1,w)),m&&w&&m.from==w.from&&m.to==w.to)return v(y(n.from,n.to,m));{let k=m?y(n.from,null,m):x(d,!1),C=w?y(null,n.to,w):x(p,!0),b=[];return(m||d).to<(w||p).from-(m&&w?1:0)||d.widgetLineBreaks>1&&k.bottom+t.defaultLineHeight/2<C.top?b.push(g(h,k.bottom,f,C.top)):k.bottom<C.top&&t.elementAtHeight((k.bottom+C.top)/2).type==Ge.Text&&(k.bottom=C.top=(k.bottom+C.top)/2),v(k).concat(b).concat(v(C))}function g(k,C,b,I){return new _r(e,k-a.left,C-a.top,Math.max(0,b-k),I-C)}function v({top:k,bottom:C,horizontal:b}){let I=[];for(let W=0;W<b.length;W+=2)I.push(g(b[W],k,b[W+1],C));return I}function y(k,C,b){let I=1e9,W=-1e9,N=[];function q($,Q,de,ae,T){let A=t.coordsAtPos($,$==b.to?-2:2),P=t.coordsAtPos(de,de==b.from?2:-2);!A||!P||(I=Math.min(A.top,P.top,I),W=Math.max(A.bottom,P.bottom,W),T==ue.LTR?N.push(s&&Q?h:A.left,s&&ae?f:P.right):N.push(!s&&ae?h:P.left,!s&&Q?f:A.right))}let j=k??b.from,M=C??b.to;for(let $ of t.visibleRanges)if($.to>j&&$.from<M)for(let Q=Math.max($.from,j),de=Math.min($.to,M);;){let ae=t.state.doc.lineAt(Q);for(let T of t.bidiSpans(ae)){let A=T.from+ae.from,P=T.to+ae.from;if(A>=de)break;P>Q&&q(Math.max(A,Q),k==null&&A<=j,Math.min(P,de),C==null&&P>=M,T.dir)}if(Q=ae.to+1,Q>=de)break}return N.length==0&&q(j,k==null,M,C==null,t.textDirection),{top:I,bottom:W,horizontal:N}}function x(k,C){let b=l.top+(C?k.top:k.bottom);return{top:b,bottom:b,horizontal:[]}}}function TR(t,e){return t.constructor==e.constructor&&t.eq(e)}class RR{constructor(e,n){this.view=e,this.layer=n,this.drawn=[],this.scaleX=1,this.scaleY=1,this.measureReq={read:this.measure.bind(this),write:this.draw.bind(this)},this.dom=e.scrollDOM.appendChild(document.createElement("div")),this.dom.classList.add("cm-layer"),n.above&&this.dom.classList.add("cm-layer-above"),n.class&&this.dom.classList.add(n.class),this.scale(),this.dom.setAttribute("aria-hidden","true"),this.setOrder(e.state),e.requestMeasure(this.measureReq),n.mount&&n.mount(this.dom,e)}update(e){e.startState.facet(_l)!=e.state.facet(_l)&&this.setOrder(e.state),(this.layer.update(e,this.dom)||e.geometryChanged)&&(this.scale(),e.view.requestMeasure(this.measureReq))}docViewUpdate(e){this.layer.updateOnDocViewUpdate!==!1&&e.requestMeasure(this.measureReq)}setOrder(e){let n=0,r=e.facet(_l);for(;n<r.length&&r[n]!=this.layer;)n++;this.dom.style.zIndex=String((this.layer.above?150:-1)-n)}measure(){return this.layer.markers(this.view)}scale(){let{scaleX:e,scaleY:n}=this.view;(e!=this.scaleX||n!=this.scaleY)&&(this.scaleX=e,this.scaleY=n,this.dom.style.transform=`scale(${1/e}, ${1/n})`)}draw(e){if(e.length!=this.drawn.length||e.some((n,r)=>!TR(n,this.drawn[r]))){let n=this.dom.firstChild,r=0;for(let i of e)i.update&&n&&i.constructor&&this.drawn[r].constructor&&i.update(n,this.drawn[r])?(n=n.nextSibling,r++):this.dom.insertBefore(i.draw(),n);for(;n;){let i=n.nextSibling;n.remove(),n=i}this.drawn=e,_.webkit&&(this.dom.style.display=this.dom.firstChild?"":"none")}}destroy(){this.layer.destroy&&this.layer.destroy(this.dom,this.view),this.dom.remove()}}const _l=z.define();function bw(t){return[Me.define(e=>new RR(e,t)),_l.of(t)]}const Gh=!_.ios,Gi=z.define({combine(t){return Zt(t,{cursorBlinkRate:1200,drawRangeCursor:!0,iosSelectionHandles:!0},{cursorBlinkRate:(e,n)=>Math.min(e,n),drawRangeCursor:(e,n)=>e||n})}});function MR(t={}){return[Gi.of(t),IR,AR,ER,Yy.of(!0)]}function Tw(t){return t.startState.facet(Gi)!=t.state.facet(Gi)}const IR=bw({above:!0,markers(t){let{state:e}=t,n=e.facet(Gi),r=[];for(let i of e.selection.ranges){let s=i==e.selection.main;if(i.empty||n.drawRangeCursor&&!(s&&_.ios&&n.iosSelectionHandles)){let o=s?"cm-cursor cm-cursor-primary":"cm-cursor cm-cursor-secondary",l=i.empty?i:R.cursor(i.head,i.assoc);for(let a of _r.forRange(t,o,l))r.push(a)}}return r},update(t,e){t.transactions.some(r=>r.selection)&&(e.style.animationName=e.style.animationName=="cm-blink"?"cm-blink2":"cm-blink");let n=Tw(t);return n&&Ym(t.state,e),t.docChanged||t.selectionSet||n},mount(t,e){Ym(e.state,t)},class:"cm-cursorLayer"});function Ym(t,e){e.style.animationDuration=t.facet(Gi).cursorBlinkRate+"ms"}const AR=bw({above:!1,markers(t){let e=[],{main:n,ranges:r}=t.state.selection;for(let i of r)if(!i.empty&&(Gh||i!=n))for(let s of _r.forRange(t,"cm-selectionBackground",i))e.push(s);if(_.ios&&!n.empty&&t.state.facet(Gi).iosSelectionHandles){for(let i of _r.forRange(t,"cm-selectionHandle cm-selectionHandle-start",R.cursor(n.from,1)))e.push(i);for(let i of _r.forRange(t,"cm-selectionHandle cm-selectionHandle-end",R.cursor(n.to,1)))e.push(i)}return e},update(t,e){return t.docChanged||t.selectionSet||t.viewportChanged||Tw(t)},class:"cm-selectionLayer"}),PR=_.gecko&&_.gecko_version==153?"#ffffff01":"transparent",ER=Zr.highest(F.theme({".cm-line":{...Gh?{"& ::selection, &::selection":{backgroundColor:`${PR} !important`}}:{},caretColor:"transparent !important"},".cm-content":{caretColor:"transparent !important","& :focus":{caretColor:"initial !important",...Gh?{"&::selection, & ::selection":{backgroundColor:"Highlight !important"}}:{}}}})),Rw=K.define({map(t,e){return t==null?null:e.mapPos(t)}}),Ts=ze.define({create(){return null},update(t,e){return t!=null&&(t=e.changes.mapPos(t)),e.effects.reduce((n,r)=>r.is(Rw)?r.value:n,t)}}),LR=Me.fromClass(class{constructor(t){this.view=t,this.cursor=null,this.measureReq={read:this.readPos.bind(this),write:this.drawCursor.bind(this)}}update(t){var e;let n=t.state.field(Ts);n==null?this.cursor!=null&&((e=this.cursor)===null||e===void 0||e.remove(),this.cursor=null):(this.cursor||(this.cursor=this.view.scrollDOM.appendChild(document.createElement("div")),this.cursor.className="cm-dropCursor"),(t.startState.field(Ts)!=n||t.docChanged||t.geometryChanged)&&this.view.requestMeasure(this.measureReq))}readPos(){let{view:t}=this,e=t.state.field(Ts),n=e!=null&&t.coordsAtPos(e);if(!n)return null;let r=t.scrollDOM.getBoundingClientRect();return{left:n.left-r.left+t.scrollDOM.scrollLeft*t.scaleX,top:n.top-r.top+t.scrollDOM.scrollTop*t.scaleY,height:n.bottom-n.top}}drawCursor(t){if(this.cursor){let{scaleX:e,scaleY:n}=this.view;t?(this.cursor.style.left=t.left/e+"px",this.cursor.style.top=t.top/n+"px",this.cursor.style.height=t.height/n+"px"):this.cursor.style.left="-100000px"}}destroy(){this.cursor&&this.cursor.remove()}setDropPos(t){this.view.state.field(Ts)!=t&&this.view.dispatch({effects:Rw.of(t)})}},{eventObservers:{dragover(t){this.setDropPos(this.view.posAtCoords({x:t.clientX,y:t.clientY}))},dragleave(t){(t.target==this.view.contentDOM||!this.view.contentDOM.contains(t.relatedTarget))&&this.setDropPos(null)},dragend(){this.setDropPos(null)},drop(){this.setDropPos(null)}}});function DR(){return[Ts,LR]}function Xm(t,e,n,r,i){e.lastIndex=0;for(let s=t.iterRange(n,r),o=n,l;!s.next().done;o+=s.value.length)if(!s.lineBreak)for(;l=e.exec(s.value);)i(o+l.index,l)}function OR(t,e){let n=t.visibleRanges;if(n.length==1&&n[0].from==t.viewport.from&&n[0].to==t.viewport.to)return n;let r=[];for(let{from:i,to:s}of n)i=Math.max(t.state.doc.lineAt(i).from,i-e),s=Math.min(t.state.doc.lineAt(s).to,s+e),r.length&&r[r.length-1].to>=i?r[r.length-1].to=s:r.push({from:i,to:s});return r}class NR{constructor(e){const{regexp:n,decoration:r,decorate:i,boundary:s,maxLength:o=1e3}=e;if(!n.global)throw new RangeError("The regular expression given to MatchDecorator should have its 'g' flag set");if(this.regexp=n,i)this.addMatch=(l,a,c,u)=>i(u,c,c+l[0].length,l,a);else if(typeof r=="function")this.addMatch=(l,a,c,u)=>{let h=r(l,a,c);h&&u(c,c+l[0].length,h)};else if(r)this.addMatch=(l,a,c,u)=>u(c,c+l[0].length,r);else throw new RangeError("Either 'decorate' or 'decoration' should be provided to MatchDecorator");this.boundary=s,this.maxLength=o}createDeco(e){let n=new xn,r=n.add.bind(n);for(let{from:i,to:s}of OR(e,this.maxLength))Xm(e.state.doc,this.regexp,i,s,(o,l)=>this.addMatch(l,e,o,r));return n.finish()}updateDeco(e,n){let r=1e9,i=-1;return e.docChanged&&e.changes.iterChanges((s,o,l,a)=>{a>=e.view.viewport.from&&l<=e.view.viewport.to&&(r=Math.min(l,r),i=Math.max(a,i))}),e.viewportMoved||i-r>1e3?this.createDeco(e.view):i>-1?this.updateRange(e.view,n.map(e.changes),r,i):n}updateRange(e,n,r,i){for(let s of e.visibleRanges){let o=Math.max(s.from,r),l=Math.min(s.to,i);if(l>=o){let a=e.state.doc.lineAt(o),c=a.to<l?e.state.doc.lineAt(l):a,u=Math.max(s.from,a.from),h=Math.min(s.to,c.to);if(this.boundary){for(;o>a.from;o--)if(this.boundary.test(a.text[o-1-a.from])){u=o;break}for(;l<c.to;l++)if(this.boundary.test(c.text[l-c.from])){h=l;break}}let f=[],d,p=(m,w,g)=>f.push(g.range(m,w));if(a==c)for(this.regexp.lastIndex=u-a.from;(d=this.regexp.exec(a.text))&&d.index<h-a.from;)this.addMatch(d,e,d.index+a.from,p);else Xm(e.state.doc,this.regexp,u,h,(m,w)=>this.addMatch(w,e,m,p));n=n.update({filterFrom:u,filterTo:h,filter:(m,w)=>m<u||w>h,add:f})}}return n}}const Kh=/x/.unicode!=null?"gu":"g",BR=new RegExp(`[\0-\b
--­؜​‎‏\u2028\u2029‭‮⁦⁧⁩\uFEFF￹-￼]`,Kh),_R={0:"null",7:"bell",8:"backspace",10:"newline",11:"vertical tab",13:"carriage return",27:"escape",8203:"zero width space",8204:"zero width non-joiner",8205:"zero width joiner",8206:"left-to-right mark",8207:"right-to-left mark",8232:"line separator",8237:"left-to-right override",8238:"right-to-left override",8294:"left-to-right isolate",8295:"right-to-left isolate",8297:"pop directional isolate",8233:"paragraph separator",65279:"zero width no-break space",65532:"object replacement"};let tu=null;function qR(){var t;if(tu==null&&typeof document<"u"&&document.body){let e=document.body.style;tu=((t=e.tabSize)!==null&&t!==void 0?t:e.MozTabSize)!=null}return tu||!1}const ql=z.define({combine(t){let e=Zt(t,{render:null,specialChars:BR,addSpecialChars:null});return(e.replaceTabs=!qR())&&(e.specialChars=new RegExp("	|"+e.specialChars.source,Kh)),e.addSpecialChars&&(e.specialChars=new RegExp(e.specialChars.source+"|"+e.addSpecialChars.source,Kh)),e}});function WR(t={}){return[ql.of(t),FR()]}let Jm=null;function FR(){return Jm||(Jm=Me.fromClass(class{constructor(t){this.view=t,this.decorations=U.none,this.decorationCache=Object.create(null),this.decorator=this.makeDecorator(t.state.facet(ql)),this.decorations=this.decorator.createDeco(t)}makeDecorator(t){return new NR({regexp:t.specialChars,decoration:(e,n,r)=>{let{doc:i}=n.state,s=ft(e[0],0);if(s==9){let o=i.lineAt(r),l=n.state.tabSize,a=es(o.text,l,r-o.from);return U.replace({widget:new HR((l-a%l)*this.view.defaultCharacterWidth/this.view.scaleX)})}return this.decorationCache[s]||(this.decorationCache[s]=U.replace({widget:new VR(t,s)}))},boundary:t.replaceTabs?void 0:/[^]/})}update(t){let e=t.state.facet(ql);t.startState.facet(ql)!=e?(this.decorator=this.makeDecorator(e),this.decorations=this.decorator.createDeco(t.view)):this.decorations=this.decorator.updateDeco(t,this.decorations)}},{decorations:t=>t.decorations}))}const zR="•";function jR(t){return t>=32?zR:t==10?"␤":String.fromCharCode(9216+t)}class VR extends qn{constructor(e,n){super(),this.options=e,this.code=n}eq(e){return e.code==this.code}toDOM(e){let n=jR(this.code),r=e.state.phrase("Control character")+" "+(_R[this.code]||"0x"+this.code.toString(16)),i=this.options.render&&this.options.render(this.code,r,n);if(i)return i;let s=document.createElement("span");return s.textContent=n,s.title=r,s.setAttribute("aria-label",r),s.className="cm-specialChar",s}ignoreEvent(){return!1}}class HR extends qn{constructor(e){super(),this.width=e}eq(e){return e.width==this.width}toDOM(){let e=document.createElement("span");return e.textContent="	",e.className="cm-tab",e.style.width=this.width+"px",e}ignoreEvent(){return!1}}function UR(){return GR}const $R=U.line({class:"cm-activeLine"}),GR=Me.fromClass(class{constructor(t){this.decorations=this.getDeco(t)}update(t){(t.docChanged||t.selectionSet)&&(this.decorations=this.getDeco(t.view))}getDeco(t){let e=-1,n=[];for(let r of t.state.selection.ranges){let i=t.lineBlockAt(r.head);i.from>e&&(n.push($R.range(i.from)),e=i.from)}return U.set(n)}},{decorations:t=>t.decorations}),Qh=2e3;function KR(t,e,n){let r=Math.min(e.line,n.line),i=Math.max(e.line,n.line),s=[];if(e.off>Qh||n.off>Qh||e.col<0||n.col<0){let o=Math.min(e.off,n.off),l=Math.max(e.off,n.off);for(let a=r;a<=i;a++){let c=t.doc.line(a);c.length<=l&&s.push(R.range(c.from+o,c.to+l))}}else{let o=Math.min(e.col,n.col),l=Math.max(e.col,n.col);for(let a=r;a<=i;a++){let c=t.doc.line(a),u=Rh(c.text,o,t.tabSize,!0);if(u<0)s.push(R.cursor(c.to));else{let h=Rh(c.text,l,t.tabSize);s.push(R.range(c.from+u,c.from+h))}}}return s}function QR(t,e){let n=t.coordsAtPos(t.viewport.from);return n?Math.round(Math.abs((n.left-e)/t.defaultCharacterWidth)):-1}function Zm(t,e){let n=t.posAtCoords({x:e.clientX,y:e.clientY},!1),r=t.state.doc.lineAt(n),i=n-r.from,s=i>Qh?-1:i==r.length?QR(t,e.clientX):es(r.text,t.state.tabSize,n-r.from);return{line:r.number,col:s,off:i}}function YR(t,e){let n=Zm(t,e),r=t.state.selection;return n?{update(i){if(i.docChanged){let s=i.changes.mapPos(i.startState.doc.line(n.line).from),o=i.state.doc.lineAt(s);n={line:o.number,col:n.col,off:Math.min(n.off,o.length)},r=r.map(i.changes)}},get(i,s,o){let l=Zm(t,i);if(!l)return r;let a=KR(t.state,n,l);return a.length?o?R.create(a.concat(r.ranges)):R.create(a):r}}:null}function XR(t){let e=n=>n.altKey&&n.button==0;return F.mouseSelectionStyle.of((n,r)=>e(r)?YR(n,r):null)}const JR={Alt:[18,t=>!!t.altKey],Control:[17,t=>!!t.ctrlKey],Shift:[16,t=>!!t.shiftKey],Meta:[91,t=>!!t.metaKey]},ZR={style:"cursor: crosshair"};function eM(t={}){let[e,n]=JR[t.key||"Alt"],r=Me.fromClass(class{constructor(i){this.view=i,this.isDown=!1}set(i){this.isDown!=i&&(this.isDown=i,this.view.update([]))}},{eventObservers:{keydown(i){this.set(i.keyCode==e||n(i))},keyup(i){(i.keyCode==e||!n(i))&&this.set(!1)},mousemove(i){this.set(n(i))}}});return[r,F.contentAttributes.of(i=>{var s;return!((s=i.plugin(r))===null||s===void 0)&&s.isDown?ZR:null})]}const cl="-10000px";class Mw{constructor(e,n,r,i){this.facet=n,this.createTooltipView=r,this.removeTooltipView=i,this.input=e.state.facet(n),this.tooltips=this.input.filter(o=>o);let s=null;this.tooltipViews=this.tooltips.map(o=>s=r(o,s))}update(e,n){var r;let i=e.state.facet(this.facet),s=i.filter(a=>a);if(i===this.input){for(let a of this.tooltipViews)a.update&&a.update(e);return!1}let o=[],l=n?[]:null;for(let a=0;a<s.length;a++){let c=s[a],u=-1;if(c){for(let h=0;h<this.tooltips.length;h++){let f=this.tooltips[h];f&&f.create==c.create&&(u=h)}if(u<0)o[a]=this.createTooltipView(c,a?o[a-1]:null),l&&(l[a]=!!c.above);else{let h=o[a]=this.tooltipViews[u];l&&(l[a]=n[u]),h.update&&h.update(e)}}}for(let a of this.tooltipViews)o.indexOf(a)<0&&(this.removeTooltipView(a),(r=a.destroy)===null||r===void 0||r.call(a));return n&&(l.forEach((a,c)=>n[c]=a),n.length=l.length),this.input=i,this.tooltips=s,this.tooltipViews=o,!0}}function Iw(t){let e=t.dom.ownerDocument.documentElement,n=e.clientWidth,r=getComputedStyle(e);if(!/stable/.test(r.scrollbarGutter||""))return{left:0,right:n};let i=e.getBoundingClientRect(),s=t.win.scrollX;return{left:Math.max(0,i.left+s-parseFloat(r.marginLeft)),right:Math.min(n,i.right+s+parseFloat(r.marginRight))}}function tM(t){let{left:e,right:n}=Iw(t);return{top:0,left:e,right:n,bottom:t.dom.ownerDocument.documentElement.clientHeight}}const nu=z.define({combine:t=>{var e,n,r;return{position:_.ios?"absolute":((e=t.find(i=>i.position))===null||e===void 0?void 0:e.position)||"fixed",parent:((n=t.find(i=>i.parent))===null||n===void 0?void 0:n.parent)||null,tooltipSpace:((r=t.find(i=>i.tooltipSpace))===null||r===void 0?void 0:r.tooltipSpace)||tM}}}),eg=new WeakMap,kd=Me.fromClass(class{constructor(t){this.view=t,this.above=[],this.inView=!0,this.madeAbsolute=!1,this.lastTransaction=0,this.measureTimeout=-1;let e=t.state.facet(nu);this.position=e.position,this.parent=e.parent,this.classes=t.themeClasses,this.createContainer(),this.measureReq={read:this.readMeasure.bind(this),write:this.writeMeasure.bind(this),key:this},this.resizeObserver=typeof ResizeObserver=="function"?new ResizeObserver(()=>this.measureSoon()):null,this.manager=new Mw(t,ic,(n,r)=>this.createTooltip(n,r),n=>{this.resizeObserver&&this.resizeObserver.unobserve(n.dom),n.dom.remove()}),this.above=this.manager.tooltips.map(n=>!!n.above),this.intersectionObserver=typeof IntersectionObserver=="function"?new IntersectionObserver(n=>{Date.now()>this.lastTransaction-50&&n.length>0&&n[n.length-1].intersectionRatio<1&&this.measureSoon()},{threshold:[1]}):null,this.observeIntersection(),t.win.addEventListener("resize",this.measureSoon=this.measureSoon.bind(this)),this.maybeMeasure()}createContainer(){this.parent?(this.container=document.createElement("div"),this.container.style.position="relative",this.container.className=this.view.themeClasses,this.parent.appendChild(this.container)):this.container=this.view.dom}observeIntersection(){if(this.intersectionObserver){this.intersectionObserver.disconnect();for(let t of this.manager.tooltipViews)this.intersectionObserver.observe(t.dom)}}measureSoon(){this.measureTimeout<0&&(this.measureTimeout=setTimeout(()=>{this.measureTimeout=-1,this.maybeMeasure()},50))}update(t){t.transactions.length&&(this.lastTransaction=Date.now());let e=this.manager.update(t,this.above);e&&this.observeIntersection();let n=e||t.geometryChanged,r=t.state.facet(nu);if(r.position!=this.position&&!this.madeAbsolute){this.position=r.position;for(let i of this.manager.tooltipViews)i.dom.style.position=this.position;n=!0}if(r.parent!=this.parent){this.parent&&this.container.remove(),this.parent=r.parent,this.createContainer();for(let i of this.manager.tooltipViews)this.container.appendChild(i.dom);n=!0}else this.parent&&this.view.themeClasses!=this.classes&&(this.classes=this.container.className=this.view.themeClasses);n&&this.maybeMeasure()}createTooltip(t,e){let n=t.create(this.view),r=e?e.dom:null;if(n.dom.classList.add("cm-tooltip"),t.arrow&&!n.dom.querySelector(".cm-tooltip > .cm-tooltip-arrow")){let i=document.createElement("div");i.className="cm-tooltip-arrow",n.dom.appendChild(i)}return n.dom.style.position=this.position,n.dom.style.top=cl,n.dom.style.left="0px",this.container.insertBefore(n.dom,r),n.mount&&n.mount(this.view),this.resizeObserver&&this.resizeObserver.observe(n.dom),n}destroy(){var t,e,n;this.view.win.removeEventListener("resize",this.measureSoon);for(let r of this.manager.tooltipViews)r.dom.remove(),(t=r.destroy)===null||t===void 0||t.call(r);this.parent&&this.container.remove(),(e=this.resizeObserver)===null||e===void 0||e.disconnect(),(n=this.intersectionObserver)===null||n===void 0||n.disconnect(),clearTimeout(this.measureTimeout)}readMeasure(){let t=1,e=1,n=!1,r=0;if(this.position=="fixed"&&this.manager.tooltipViews.length){let{dom:o}=this.manager.tooltipViews[0];if(_.safari){let l=o.getBoundingClientRect();n=Math.abs(l.top+1e4)>1||Math.abs(l.left)>1}else n=!!o.offsetParent&&o.offsetParent!=this.container.ownerDocument.body}if(n||this.position=="absolute")if(this.parent){let o=this.parent.getBoundingClientRect();o.width&&o.height&&(t=o.width/this.parent.offsetWidth,e=o.height/this.parent.offsetHeight)}else({scaleX:t,scaleY:e}=this.view.viewState);else r=Iw(this.view).left;let i=this.view.scrollDOM.getBoundingClientRect(),s=vd(this.view);return{visible:{left:i.left+s.left,top:i.top+s.top,right:i.right-s.right,bottom:i.bottom-s.bottom},parent:this.parent?this.container.getBoundingClientRect():this.view.dom.getBoundingClientRect(),pos:this.manager.tooltips.map((o,l)=>{let a=this.manager.tooltipViews[l];return a.getCoords?a.getCoords(o.pos):this.view.coordsAtPos(o.pos)}),size:this.manager.tooltipViews.map(({dom:o})=>o.getBoundingClientRect()),space:this.view.state.facet(nu).tooltipSpace(this.view),scaleX:t,scaleY:e,makeAbsolute:n,fixedLeft:r}}writeMeasure(t){var e;if(t.makeAbsolute){this.madeAbsolute=!0,this.position="absolute";for(let l of this.manager.tooltipViews)l.dom.style.position="absolute"}let{visible:n,space:r,scaleX:i,scaleY:s}=t,o=[];for(let l=0;l<this.manager.tooltips.length;l++){let a=this.manager.tooltips[l],c=this.manager.tooltipViews[l],{dom:u}=c,h=t.pos[l],f=t.size[l];if(!h||a.clip!==!1&&(h.bottom<=Math.max(n.top,r.top)||h.top>=Math.min(n.bottom,r.bottom)||h.right<Math.max(n.left,r.left)-.1||h.left>Math.min(n.right,r.right)+.1)){u.style.top=cl;continue}let d=a.arrow?c.dom.querySelector(".cm-tooltip-arrow"):null,p=d?7:0,m=f.right-f.left,w=(e=eg.get(c))!==null&&e!==void 0?e:f.bottom-f.top,g=c.offset||rM,v=this.view.textDirection==ue.LTR,y=f.width>r.right-r.left?v?r.left:r.right-f.width:v?Math.max(r.left,Math.min(h.left-(d?14:0)+g.x,r.right-m)):Math.min(Math.max(r.left,h.left-m+(d?14:0)-g.x),r.right-m),x=this.above[l];!a.strictSide&&(x?h.top-w-p-g.y<r.top:h.bottom+w+p+g.y>r.bottom)&&x==r.bottom-h.bottom>h.top-r.top&&(x=this.above[l]=!x);let k=(x?h.top-r.top:r.bottom-h.bottom)-p;if(k<w&&c.resize!==!1){if(k<this.view.defaultLineHeight){u.style.top=cl;continue}eg.set(c,w),u.style.height=(w=k)/s+"px"}else u.style.height&&(u.style.height="");let C=x?h.top-w-p-g.y:h.bottom+p+g.y,b=y+m;if(c.overlap!==!0)for(let I of o)I.left<b&&I.right>y&&I.top<C+w&&I.bottom>C&&(C=x?I.top-w-2-p:I.bottom+p+2);if(this.position=="absolute"?(u.style.top=(C-t.parent.top)/s+"px",tg(u,(y-t.parent.left)/i)):(u.style.top=C/s+"px",tg(u,(y-t.fixedLeft)/i)),d){let I=h.left+(v?g.x:-g.x)-(y+14-7);d.style.left=I/i+"px"}c.overlap!==!0&&o.push({left:y,top:C,right:b,bottom:C+w}),u.classList.toggle("cm-tooltip-above",x),u.classList.toggle("cm-tooltip-below",!x),c.positioned&&c.positioned(t.space)}}maybeMeasure(){if(this.manager.tooltips.length&&(this.view.inView&&this.view.requestMeasure(this.measureReq),this.inView!=this.view.inView&&(this.inView=this.view.inView,!this.inView)))for(let t of this.manager.tooltipViews)t.dom.style.top=cl}},{eventObservers:{scroll(){this.maybeMeasure()}}});function tg(t,e){let n=parseInt(t.style.left,10);(isNaN(n)||Math.abs(e-n)>1)&&(t.style.left=e+"px")}const nM=F.baseTheme({".cm-tooltip":{zIndex:500,boxSizing:"border-box"},"&light .cm-tooltip":{border:"1px solid #bbb",backgroundColor:"#f5f5f5"},"&light .cm-tooltip-section:not(:first-child)":{borderTop:"1px solid #bbb"},"&dark .cm-tooltip":{backgroundColor:"#333338",color:"white"},".cm-tooltip-arrow":{height:"7px",width:`${7*2}px`,position:"absolute",zIndex:-1,overflow:"hidden","&:before, &:after":{content:"''",position:"absolute",width:0,height:0,borderLeft:"7px solid transparent",borderRight:"7px solid transparent"},".cm-tooltip-above &":{bottom:"-7px","&:before":{borderTop:"7px solid #bbb"},"&:after":{borderTop:"7px solid #f5f5f5",bottom:"1px"}},".cm-tooltip-below &":{top:"-7px","&:before":{borderBottom:"7px solid #bbb"},"&:after":{borderBottom:"7px solid #f5f5f5",top:"1px"}}},"&dark .cm-tooltip .cm-tooltip-arrow":{"&:before":{borderTopColor:"#333338",borderBottomColor:"#333338"},"&:after":{borderTopColor:"transparent",borderBottomColor:"transparent"}}}),rM={x:0,y:0},ic=z.define({enables:[kd,nM]}),Ca=z.define({combine:t=>t.reduce((e,n)=>e.concat(n),[])});class sc{static create(e){return new sc(e)}constructor(e){this.view=e,this.mounted=!1,this.dom=document.createElement("div"),this.dom.classList.add("cm-tooltip-hover"),this.manager=new Mw(e,Ca,(n,r)=>this.createHostedView(n,r),n=>n.dom.remove())}createHostedView(e,n){let r=e.create(this.view);return r.dom.classList.add("cm-tooltip-section"),this.dom.insertBefore(r.dom,n?n.dom.nextSibling:this.dom.firstChild),this.mounted&&r.mount&&r.mount(this.view),r}mount(e){for(let n of this.manager.tooltipViews)n.mount&&n.mount(e);this.mounted=!0}positioned(e){for(let n of this.manager.tooltipViews)n.positioned&&n.positioned(e)}update(e){this.manager.update(e)}destroy(){var e;for(let n of this.manager.tooltipViews)(e=n.destroy)===null||e===void 0||e.call(n)}passProp(e){let n;for(let r of this.manager.tooltipViews){let i=r[e];if(i!==void 0){if(n===void 0)n=i;else if(n!==i)return}}return n}get offset(){return this.passProp("offset")}get getCoords(){return this.passProp("getCoords")}get overlap(){return this.passProp("overlap")}get resize(){return this.passProp("resize")}}const iM=ic.compute([Ca],t=>{let e=t.facet(Ca);return e.length===0?null:{pos:Math.min(...e.map(n=>n.pos)),end:Math.max(...e.map(n=>{var r;return(r=n.end)!==null&&r!==void 0?r:n.pos})),create:sc.create,above:e[0].above,arrow:e.some(n=>n.arrow)}}),Aw=z.define();class sM{constructor(e,n,r,i,s,o){this.view=e,this.source=n,this.field=r,this.locked=i,this.setHover=s,this.hoverTime=o,this.hoverTimeout=-1,this.restartTimeout=-1,this.pending=null,this.lastMove={x:0,y:0,target:e.dom,time:0},this.checkHover=this.checkHover.bind(this),e.dom.addEventListener("mouseleave",this.mouseleave=this.mouseleave.bind(this)),e.dom.addEventListener("mousemove",this.mousemove=this.mousemove.bind(this))}update(e){this.pending&&(this.pending=null,clearTimeout(this.restartTimeout),this.restartTimeout=setTimeout(()=>this.startHover(),20))}get active(){return this.view.state.field(this.field)}checkHover(){if(this.hoverTimeout=-1,this.active.length)return;let e=Date.now()-this.lastMove.time;e<this.hoverTime?this.hoverTimeout=setTimeout(this.checkHover,this.hoverTime-e):this.startHover()}startHover(){clearTimeout(this.restartTimeout);let{view:e,lastMove:n}=this,r=e.docView.tile.nearest(n.target);if(!r)return;let i,s=1;if(r.isWidget())i=r.posAtStart;else{if(i=e.posAtCoords(n),i==null)return;let o=e.coordsAtPos(i);if(!o||n.y<o.top||n.y>o.bottom||n.x<o.left-e.defaultCharacterWidth||n.x>o.right+e.defaultCharacterWidth)return;let l=e.bidiSpans(e.state.doc.lineAt(i)).find(c=>c.from<=i&&c.to>=i),a=l&&l.dir==ue.RTL?-1:1;s=n.x<o.left?-a:a}this.activateHover(e,i,s)}activateHover(e,n,r,i){let s=this.source(e,n,r),o=l=>{if(l&&!(Array.isArray(l)&&!l.length)){let a=Array.isArray(l)?l:[l];i&&this.locked.set(a,i),e.dispatch({effects:this.setHover.of(a)})}};if(s&&"then"in s){let l=this.pending={pos:n};s.then(a=>{this.pending==l&&(this.pending=null,o(a))},a=>st(e.state,a,"hover tooltip"))}else o(s)}get tooltip(){let e=this.view.plugin(kd),n=e?e.manager.tooltips.findIndex(r=>r.create==sc.create):-1;return n>-1?e.manager.tooltipViews[n]:null}mousemove(e){var n,r;this.lastMove={x:e.clientX,y:e.clientY,target:e.target,time:Date.now()},this.hoverTimeout<0&&(this.hoverTimeout=setTimeout(this.checkHover,this.hoverTime));let{active:i,tooltip:s}=this;if(i.length&&!this.locked.has(i)&&s&&!oM(s.dom,e)||this.pending){let{pos:o}=i[0]||this.pending,l=(r=(n=i[0])===null||n===void 0?void 0:n.end)!==null&&r!==void 0?r:o;(o==l?this.view.posAtCoords(this.lastMove)!=o:!lM(this.view,o,l,e.clientX,e.clientY))&&(this.view.dispatch({effects:this.setHover.of([])}),this.pending=null)}}mouseleave(e){clearTimeout(this.hoverTimeout),this.hoverTimeout=-1;let{active:n}=this;if(n.length&&!this.locked.has(n)){let{tooltip:r}=this;r&&r.dom.contains(e.relatedTarget)?this.watchTooltipLeave(r.dom):this.view.dispatch({effects:this.setHover.of([])})}}watchTooltipLeave(e){let n=r=>{e.removeEventListener("mouseleave",n);let{active:i}=this;i.length&&!this.locked.has(i)&&!this.view.dom.contains(r.relatedTarget)&&this.view.dispatch({effects:this.setHover.of([])})};e.addEventListener("mouseleave",n)}destroy(){clearTimeout(this.hoverTimeout),clearTimeout(this.restartTimeout),this.view.dom.removeEventListener("mouseleave",this.mouseleave),this.view.dom.removeEventListener("mousemove",this.mousemove)}}const ul=4;function oM(t,e){let{left:n,right:r,top:i,bottom:s}=t.getBoundingClientRect(),o;if(o=t.querySelector(".cm-tooltip-arrow")){let l=o.getBoundingClientRect();i=Math.min(l.top,i),s=Math.max(l.bottom,s)}return e.clientX>=n-ul&&e.clientX<=r+ul&&e.clientY>=i-ul&&e.clientY<=s+ul}function lM(t,e,n,r,i,s){let o=t.scrollDOM.getBoundingClientRect(),l=t.documentTop+t.documentPadding.top+t.contentHeight;if(o.left>r||o.right<r||o.top>i||Math.min(o.bottom,l)<i)return!1;let a=t.posAtCoords({x:r,y:i},!1);return a>=e&&a<=n}function aM(t,e={}){let n=K.define(),r=new WeakMap,i=ze.define({create(){return[]},update(o,l){let a=r.get(o);if(o.length&&(e.hideOnChange&&(l.docChanged||l.selection)?o=[]:a&&a(l)?o=[]:e.hideOn&&(o=o.filter(c=>!e.hideOn(l,c)))),l.docChanged&&o.length){let c=[];for(let u of o){let h=l.changes.mapPos(u.pos,-1,it.TrackDel);if(h!=null){let f=Object.assign(Object.create(null),u);f.pos=h,f.end!=null&&(f.end=l.changes.mapPos(f.end)),c.push(f)}}o=c}for(let c of l.effects)c.is(n)&&(o=c.value,a=void 0),(c.is(uM)&&!c.value||c.value==i)&&(o=[]);return o.length&&a&&r.set(o,a),o},provide:o=>Ca.from(o)});const s=Me.define(o=>new sM(o,t,i,r,n,e.hoverTime||300));return{active:i,extension:[i,s,Aw.of(s),iM]}}function cM(t,e,n,r={}){var i;let s=t.state.facet(Aw).map(o=>t.plugin(o)).filter(o=>!!o);if(r.tooltip&&r.tooltip.active){let o=s.find(l=>l.field==r.tooltip.active);o&&(s=[o])}for(let o of s)o.activateHover(t,e,n,(i=r.until)!==null&&i!==void 0?i:()=>!1)}function Pw(t,e){let n=t.plugin(kd);if(!n)return null;let r=n.manager.tooltips.indexOf(e);return r<0?null:n.manager.tooltipViews[r]}const uM=K.define(),ng=z.define({combine(t){let e,n;for(let r of t)e=e||r.topContainer,n=n||r.bottomContainer;return{topContainer:e,bottomContainer:n}}});function oc(t,e){let n=t.plugin(Ew),r=n?n.specs.indexOf(e):-1;return r>-1?n.panels[r]:null}const Ew=Me.fromClass(class{constructor(t){this.input=t.state.facet(fo),this.specs=this.input.filter(n=>n),this.panels=this.specs.map(n=>n(t));let e=t.state.facet(ng);this.top=new hl(t,!0,e.topContainer),this.bottom=new hl(t,!1,e.bottomContainer),this.top.sync(this.panels.filter(n=>n.top)),this.bottom.sync(this.panels.filter(n=>!n.top));for(let n of this.panels)n.dom.classList.add("cm-panel"),n.mount&&n.mount()}update(t){let e=t.state.facet(ng);this.top.container!=e.topContainer&&(this.top.sync([]),this.top=new hl(t.view,!0,e.topContainer)),this.bottom.container!=e.bottomContainer&&(this.bottom.sync([]),this.bottom=new hl(t.view,!1,e.bottomContainer)),this.top.syncClasses(),this.bottom.syncClasses();let n=t.state.facet(fo);if(n!=this.input){let r=n.filter(a=>a),i=[],s=[],o=[],l=[];for(let a of r){let c=this.specs.indexOf(a),u;c<0?(u=a(t.view),l.push(u)):(u=this.panels[c],u.update&&u.update(t)),i.push(u),(u.top?s:o).push(u)}this.specs=r,this.panels=i,this.top.sync(s),this.bottom.sync(o);for(let a of l)a.dom.classList.add("cm-panel"),a.mount&&a.mount()}else for(let r of this.panels)r.update&&r.update(t)}destroy(){this.top.sync([]),this.bottom.sync([])}},{provide:t=>F.scrollMargins.of(e=>{let n=e.plugin(t);return n&&{top:n.top.scrollMargin(),bottom:n.bottom.scrollMargin()}})});class hl{constructor(e,n,r){this.view=e,this.top=n,this.container=r,this.dom=void 0,this.classes="",this.panels=[],this.syncClasses()}sync(e){for(let n of this.panels)n.destroy&&e.indexOf(n)<0&&n.destroy();this.panels=e,this.syncDOM()}syncDOM(){if(this.panels.length==0){this.dom&&(this.dom.remove(),this.dom=void 0);return}if(!this.dom){this.dom=document.createElement("div"),this.dom.className=this.top?"cm-panels cm-panels-top":"cm-panels cm-panels-bottom";let n=this.container||this.view.dom;n.insertBefore(this.dom,this.top?n.firstChild:null)}let e=this.dom.firstChild;for(let n of this.panels)if(n.dom.parentNode==this.dom){for(;e!=n.dom;)e=rg(e);e=e.nextSibling}else this.dom.insertBefore(n.dom,e);for(;e;)e=rg(e)}scrollMargin(){return!this.dom||this.container?0:Math.max(0,this.top?this.dom.getBoundingClientRect().bottom-Math.max(0,this.view.scrollDOM.getBoundingClientRect().top):Math.min(innerHeight,this.view.scrollDOM.getBoundingClientRect().bottom)-this.dom.getBoundingClientRect().top)}syncClasses(){if(!(!this.container||this.classes==this.view.themeClasses)){for(let e of this.classes.split(" "))e&&this.container.classList.remove(e);for(let e of(this.classes=this.view.themeClasses).split(" "))e&&this.container.classList.add(e)}}}function rg(t){let e=t.nextSibling;return t.remove(),e}const fo=z.define({enables:Ew});function hM(t,e){let n,r=new Promise(o=>n=o),i=o=>dM(o,e,n);t.state.field(Wl,!1)?t.dispatch({effects:Lw.of(i)}):t.dispatch({effects:K.appendConfig.of(Wl.init(()=>[i]))});let s=Dw.of(i);return{close:s,result:r.then(o=>((t.win.queueMicrotask||(a=>t.win.setTimeout(a,10)))(()=>{t.state.field(Wl).indexOf(i)>-1&&t.dispatch({effects:s})}),o))}}function fM(t,e){let n=t.state.field(Wl,!1)||[];for(let r of n){let i=oc(t,r);if(i&&i.dom.classList.contains(e))return i}return null}const Wl=ze.define({create(){return[]},update(t,e){for(let n of e.effects)n.is(Lw)?t=[n.value].concat(t):n.is(Dw)&&(t=t.filter(r=>r!=n.value));return t},provide:t=>fo.computeN([t],e=>e.field(t))}),Lw=K.define(),Dw=K.define();function dM(t,e,n){let r=e.content?e.content(t,()=>o(null)):null;if(!r){if(r=le("form"),e.input){let l=le("input",e.input);/^(text|password|number|email|tel|url)$/.test(l.type)&&l.classList.add("cm-textfield"),l.name||(l.name="input"),r.appendChild(le("label",(e.label||"")+": ",l))}else r.appendChild(document.createTextNode(e.label||""));r.appendChild(document.createTextNode(" ")),r.appendChild(le("button",{class:"cm-button",type:"submit"},e.submitLabel||"OK"))}let i=r.nodeName=="FORM"?[r]:r.querySelectorAll("form");for(let l=0;l<i.length;l++){let a=i[l];a.addEventListener("keydown",c=>{c.keyCode==27?(c.preventDefault(),o(null)):c.keyCode==13&&(c.preventDefault(),o(a))}),a.addEventListener("submit",c=>{c.preventDefault(),o(a)})}let s=le("div",r,le("button",{onclick:()=>o(null),"aria-label":t.state.phrase("close"),class:"cm-dialog-close",type:"button"},["×"]));e.class&&(s.className=e.class),s.classList.add("cm-dialog");function o(l){s.contains(s.ownerDocument.activeElement)&&t.focus(),n(l)}return{dom:s,top:e.top,mount:()=>{if(e.focus){let l;typeof e.focus=="string"?l=r.querySelector(e.focus):l=r.querySelector("input")||r.querySelector("button"),l&&"select"in l?l.select():l&&"focus"in l&&l.focus()}}}}class Cn extends ur{compare(e){return this==e||this.constructor==e.constructor&&this.eq(e)}eq(e){return!1}destroy(e){}}Cn.prototype.elementClass="";Cn.prototype.toDOM=void 0;Cn.prototype.mapMode=it.TrackBefore;Cn.prototype.startSide=Cn.prototype.endSide=-1;Cn.prototype.point=!0;const Fl=z.define(),pM=z.define(),mM={class:"",renderEmptyElements:!1,elementStyle:"",markers:()=>X.empty,lineMarker:()=>null,widgetMarker:()=>null,lineMarkerChange:null,initialSpacer:null,updateSpacer:null,domEventHandlers:{},side:"before"},Fs=z.define();function Ow(t){return[Nw(),Fs.of({...mM,...t})]}const ig=z.define({combine:t=>t.some(e=>e)});function Nw(t){return[gM]}const gM=Me.fromClass(class{constructor(t){this.view=t,this.domAfter=null,this.prevViewport=t.viewport,this.dom=document.createElement("div"),this.dom.className="cm-gutters cm-gutters-before",this.dom.setAttribute("aria-hidden","true"),this.dom.style.minHeight=this.view.contentHeight/this.view.scaleY+"px",this.gutters=t.state.facet(Fs).map(e=>new og(t,e)),this.fixed=!t.state.facet(ig);for(let e of this.gutters)e.config.side=="after"?this.getDOMAfter().appendChild(e.dom):this.dom.appendChild(e.dom);this.fixed&&(this.dom.style.position="sticky"),this.syncGutters(!1),t.scrollDOM.insertBefore(this.dom,t.contentDOM)}getDOMAfter(){return this.domAfter||(this.domAfter=document.createElement("div"),this.domAfter.className="cm-gutters cm-gutters-after",this.domAfter.setAttribute("aria-hidden","true"),this.domAfter.style.minHeight=this.view.contentHeight/this.view.scaleY+"px",this.domAfter.style.position=this.fixed?"sticky":"",this.view.scrollDOM.appendChild(this.domAfter)),this.domAfter}update(t){if(this.updateGutters(t)){let e=this.prevViewport,n=t.view.viewport,r=Math.min(e.to,n.to)-Math.max(e.from,n.from);this.syncGutters(r<(n.to-n.from)*.8)}if(t.geometryChanged){let e=this.view.contentHeight/this.view.scaleY+"px";this.dom.style.minHeight=e,this.domAfter&&(this.domAfter.style.minHeight=e)}this.view.state.facet(ig)!=!this.fixed&&(this.fixed=!this.fixed,this.dom.style.position=this.fixed?"sticky":"",this.domAfter&&(this.domAfter.style.position=this.fixed?"sticky":"")),this.prevViewport=t.view.viewport}syncGutters(t){let e=this.dom.nextSibling;t&&(this.dom.remove(),this.domAfter&&this.domAfter.remove());let n=X.iter(this.view.state.facet(Fl),this.view.viewport.from),r=[],i=this.gutters.map(s=>new vM(s,this.view.viewport,-this.view.documentPadding.top));for(let s of this.view.viewportLineBlocks)if(r.length&&(r=[]),Array.isArray(s.type)){let o=!0;for(let l of s.type)if(l.type==Ge.Text&&o){Yh(n,r,l.from);for(let a of i)a.line(this.view,l,r);o=!1}else if(l.widget)for(let a of i)a.widget(this.view,l)}else if(s.type==Ge.Text){Yh(n,r,s.from);for(let o of i)o.line(this.view,s,r)}else if(s.widget)for(let o of i)o.widget(this.view,s);for(let s of i)s.finish();t&&(this.view.scrollDOM.insertBefore(this.dom,e),this.domAfter&&this.view.scrollDOM.appendChild(this.domAfter))}updateGutters(t){let e=t.startState.facet(Fs),n=t.state.facet(Fs),r=t.docChanged||t.heightChanged||t.viewportChanged||!X.eq(t.startState.facet(Fl),t.state.facet(Fl),t.view.viewport.from,t.view.viewport.to);if(e==n)for(let i of this.gutters)i.update(t)&&(r=!0);else{r=!0;let i=[];for(let s of n){let o=e.indexOf(s);o<0?i.push(new og(this.view,s)):(this.gutters[o].update(t),i.push(this.gutters[o]))}for(let s of this.gutters)s.dom.remove(),i.indexOf(s)<0&&s.destroy();for(let s of i)s.config.side=="after"?this.getDOMAfter().appendChild(s.dom):this.dom.appendChild(s.dom);this.gutters=i}return r}destroy(){for(let t of this.gutters)t.destroy();this.dom.remove(),this.domAfter&&this.domAfter.remove()}},{provide:t=>F.scrollMargins.of(e=>{let n=e.plugin(t);if(!n||n.gutters.length==0||!n.fixed)return null;let r=n.dom.offsetWidth*e.scaleX,i=n.domAfter?n.domAfter.offsetWidth*e.scaleX:0;return e.textDirection==ue.LTR?{left:r,right:i}:{right:r,left:i}})});function sg(t){return Array.isArray(t)?t:[t]}function Yh(t,e,n){for(;t.value&&t.from<=n;)t.from==n&&e.push(t.value),t.next()}class vM{constructor(e,n,r){this.gutter=e,this.height=r,this.i=0,this.cursor=X.iter(e.markers,n.from)}addElement(e,n,r){let{gutter:i}=this,s=(n.top-this.height)/e.scaleY,o=n.height/e.scaleY;if(this.i==i.elements.length){let l=new Bw(e,o,s,r);i.elements.push(l),i.dom.appendChild(l.dom)}else i.elements[this.i].update(e,o,s,r);this.height=n.bottom,this.i++}line(e,n,r){let i=[];Yh(this.cursor,i,n.from),r.length&&(i=i.concat(r));let s=this.gutter.config.lineMarker(e,n,i);s&&i.unshift(s);let o=this.gutter;i.length==0&&!o.config.renderEmptyElements||this.addElement(e,n,i)}widget(e,n){let r=this.gutter.config.widgetMarker(e,n.widget,n),i=r?[r]:null;for(let s of e.state.facet(pM)){let o=s(e,n.widget,n);o&&(i||(i=[])).push(o)}i&&this.addElement(e,n,i)}finish(){let e=this.gutter;for(;e.elements.length>this.i;){let n=e.elements.pop();e.dom.removeChild(n.dom),n.destroy()}}}class og{constructor(e,n){this.view=e,this.config=n,this.elements=[],this.spacer=null,this.dom=document.createElement("div"),this.dom.className="cm-gutter"+(this.config.class?" "+this.config.class:"");for(let r in n.domEventHandlers)this.dom.addEventListener(r,i=>{let s=i.target,o;if(s!=this.dom&&this.dom.contains(s)){for(;s.parentNode!=this.dom;)s=s.parentNode;let a=s.getBoundingClientRect();o=(a.top+a.bottom)/2}else o=i.clientY;let l=e.lineBlockAtHeight(o-e.documentTop);n.domEventHandlers[r](e,l,i)&&i.preventDefault()});this.markers=sg(n.markers(e)),n.initialSpacer&&(this.spacer=new Bw(e,0,0,[n.initialSpacer(e)]),this.dom.appendChild(this.spacer.dom),this.spacer.dom.style.cssText+="visibility: hidden; pointer-events: none")}update(e){let n=this.markers;if(this.markers=sg(this.config.markers(e.view)),this.spacer&&this.config.updateSpacer){let i=this.config.updateSpacer(this.spacer.markers[0],e);i!=this.spacer.markers[0]&&this.spacer.update(e.view,0,0,[i])}let r=e.view.viewport;return!X.eq(this.markers,n,r.from,r.to)||(this.config.lineMarkerChange?this.config.lineMarkerChange(e):!1)}destroy(){for(let e of this.elements)e.destroy()}}class Bw{constructor(e,n,r,i){this.height=-1,this.above=0,this.markers=[],this.dom=document.createElement("div"),this.dom.className="cm-gutterElement",this.update(e,n,r,i)}update(e,n,r,i){this.height!=n&&(this.height=n,this.dom.style.height=n+"px"),this.above!=r&&(this.dom.style.marginTop=(this.above=r)?r+"px":""),yM(this.markers,i)||this.setMarkers(e,i)}setMarkers(e,n){let r="cm-gutterElement",i=this.dom.firstChild;for(let s=0,o=0;;){let l=o,a=s<n.length?n[s++]:null,c=!1;if(a){let u=a.elementClass;u&&(r+=" "+u);for(let h=o;h<this.markers.length;h++)if(this.markers[h].compare(a)){l=h,c=!0;break}}else l=this.markers.length;for(;o<l;){let u=this.markers[o++];if(u.toDOM){u.destroy(i);let h=i.nextSibling;i.remove(),i=h}}if(!a)break;a.toDOM&&(c?i=i.nextSibling:this.dom.insertBefore(a.toDOM(e),i)),c&&o++}this.dom.className=r,this.markers=n}destroy(){this.setMarkers(null,[])}}function yM(t,e){if(t.length!=e.length)return!1;for(let n=0;n<t.length;n++)if(!t[n].compare(e[n]))return!1;return!0}const wM=z.define(),SM=z.define(),wi=z.define({combine(t){return Zt(t,{formatNumber:String,domEventHandlers:{}},{domEventHandlers(e,n){let r=Object.assign({},e);for(let i in n){let s=r[i],o=n[i];r[i]=s?(l,a,c)=>s(l,a,c)||o(l,a,c):o}return r}})}});class ru extends Cn{constructor(e){super(),this.number=e}eq(e){return this.number==e.number}toDOM(){return document.createTextNode(this.number)}}function iu(t,e){return t.state.facet(wi).formatNumber(e,t.state)}const kM=Fs.compute([wi],t=>({class:"cm-lineNumbers",renderEmptyElements:!1,markers(e){return e.state.facet(wM)},lineMarker(e,n,r){return r.some(i=>i.toDOM)?null:new ru(iu(e,e.state.doc.lineAt(n.from).number))},widgetMarker:(e,n,r)=>{for(let i of e.state.facet(SM)){let s=i(e,n,r);if(s)return s}return null},lineMarkerChange:e=>e.startState.facet(wi)!=e.state.facet(wi),initialSpacer(e){return new ru(iu(e,lg(e.state.doc.lines)))},updateSpacer(e,n){let r=iu(n.view,lg(n.view.state.doc.lines));return r==e.number?e:new ru(r)},domEventHandlers:t.facet(wi).domEventHandlers,side:"before"}));function xM(t={}){return[wi.of(t),Nw(),kM]}function lg(t){let e=9;for(;e<t;)e=e*10+9;return e}const CM=new class extends Cn{constructor(){super(...arguments),this.elementClass="cm-activeLineGutter"}},bM=Fl.compute(["selection"],t=>{let e=[],n=-1;for(let r of t.selection.ranges){let i=t.doc.lineAt(r.head).from;i>n&&(n=i,e.push(CM.range(i)))}return X.of(e)});function TM(){return bM}const RM=1024;let MM=0;class su{constructor(e,n){this.from=e,this.to=n}}class Z{constructor(e={}){this.id=MM++,this.perNode=!!e.perNode,this.deserialize=e.deserialize||(()=>{throw new Error("This node type doesn't define a deserialize function")}),this.combine=e.combine||null}add(e){if(this.perNode)throw new RangeError("Can't add per-node props to node types");return typeof e!="function"&&(e=St.match(e)),n=>{let r=e(n);return r===void 0?null:[this,r]}}}Z.closedBy=new Z({deserialize:t=>t.split(" ")});Z.openedBy=new Z({deserialize:t=>t.split(" ")});Z.group=new Z({deserialize:t=>t.split(" ")});Z.isolate=new Z({deserialize:t=>{if(t&&t!="rtl"&&t!="ltr"&&t!="auto")throw new RangeError("Invalid value for isolate: "+t);return t||"auto"}});Z.contextHash=new Z({perNode:!0});Z.lookAhead=new Z({perNode:!0});Z.mounted=new Z({perNode:!0});class zs{constructor(e,n,r,i=!1){this.tree=e,this.overlay=n,this.parser=r,this.bracketed=i}static get(e){return e&&e.props&&e.props[Z.mounted.id]}}const IM=Object.create(null);class St{constructor(e,n,r,i=0){this.name=e,this.props=n,this.id=r,this.flags=i}static define(e){let n=e.props&&e.props.length?Object.create(null):IM,r=(e.top?1:0)|(e.skipped?2:0)|(e.error?4:0)|(e.name==null?8:0),i=new St(e.name||"",n,e.id,r);if(e.props){for(let s of e.props)if(Array.isArray(s)||(s=s(i)),s){if(s[0].perNode)throw new RangeError("Can't store a per-node prop on a node type");n[s[0].id]=s[1]}}return i}prop(e){return this.props[e.id]}get isTop(){return(this.flags&1)>0}get isSkipped(){return(this.flags&2)>0}get isError(){return(this.flags&4)>0}get isAnonymous(){return(this.flags&8)>0}is(e){if(typeof e=="string"){if(this.name==e)return!0;let n=this.prop(Z.group);return n?n.indexOf(e)>-1:!1}return this.id==e}static match(e){let n=Object.create(null);for(let r in e)for(let i of r.split(" "))n[i]=e[r];return r=>{for(let i=r.prop(Z.group),s=-1;s<(i?i.length:0);s++){let o=n[s<0?r.name:i[s]];if(o)return o}}}}St.none=new St("",Object.create(null),0,8);class xd{constructor(e){this.types=e;for(let n=0;n<e.length;n++)if(e[n].id!=n)throw new RangeError("Node type ids should correspond to array positions when creating a node set")}extend(...e){let n=[];for(let r of this.types){let i=null;for(let s of e){let o=s(r);if(o){i||(i=Object.assign({},r.props));let l=o[1],a=o[0];a.combine&&a.id in i&&(l=a.combine(i[a.id],l)),i[a.id]=l}}n.push(i?new St(r.name,i,r.id,r.flags):r)}return new xd(n)}}const fl=new WeakMap,ag=new WeakMap;var Ae;(function(t){t[t.ExcludeBuffers=1]="ExcludeBuffers",t[t.IncludeAnonymous=2]="IncludeAnonymous",t[t.IgnoreMounts=4]="IgnoreMounts",t[t.IgnoreOverlays=8]="IgnoreOverlays",t[t.EnterBracketed=16]="EnterBracketed"})(Ae||(Ae={}));class Se{constructor(e,n,r,i,s){if(this.type=e,this.children=n,this.positions=r,this.length=i,this.props=null,s&&s.length){this.props=Object.create(null);for(let[o,l]of s)this.props[typeof o=="number"?o:o.id]=l}}toString(){let e=zs.get(this);if(e&&!e.overlay)return e.tree.toString();let n="";for(let r of this.children){let i=r.toString();i&&(n&&(n+=","),n+=i)}return this.type.name?(/\W/.test(this.type.name)&&!this.type.isError?JSON.stringify(this.type.name):this.type.name)+(n.length?"("+n+")":""):n}cursor(e=0){return new Jh(this.topNode,e)}cursorAt(e,n=0,r=0){let i=fl.get(this)||this.topNode,s=new Jh(i);return s.moveTo(e,n),fl.set(this,s._tree),s}get topNode(){return new qt(this,0,0,null)}resolve(e,n=0){let r=po(fl.get(this)||this.topNode,e,n,!1);return fl.set(this,r),r}resolveInner(e,n=0){let r=po(ag.get(this)||this.topNode,e,n,!0);return ag.set(this,r),r}resolveStack(e,n=0){return EM(this,e,n)}iterate(e){let{enter:n,leave:r,from:i=0,to:s=this.length}=e,o=e.mode||0,l=(o&Ae.IncludeAnonymous)>0;for(let a=this.cursor(o|Ae.IncludeAnonymous);;){let c=!1;if(a.from<=s&&a.to>=i&&(!l&&a.type.isAnonymous||n(a)!==!1)){if(a.firstChild())continue;c=!0}for(;c&&r&&(l||!a.type.isAnonymous)&&r(a),!a.nextSibling();){if(!a.parent())return;c=!0}}}prop(e){return e.perNode?this.props?this.props[e.id]:void 0:this.type.prop(e)}get propValues(){let e=[];if(this.props)for(let n in this.props)e.push([+n,this.props[n]]);return e}balance(e={}){return this.children.length<=8?this:Td(St.none,this.children,this.positions,0,this.children.length,0,this.length,(n,r,i)=>new Se(this.type,n,r,i,this.propValues),e.makeTree||((n,r,i)=>new Se(St.none,n,r,i)))}static build(e){return LM(e)}}Se.empty=new Se(St.none,[],[],0);class Cd{constructor(e,n){this.buffer=e,this.index=n}get id(){return this.buffer[this.index-4]}get start(){return this.buffer[this.index-3]}get end(){return this.buffer[this.index-2]}get size(){return this.buffer[this.index-1]}get pos(){return this.index}next(){this.index-=4}fork(){return new Cd(this.buffer,this.index)}}class pr{constructor(e,n,r){this.buffer=e,this.length=n,this.set=r}get type(){return St.none}toString(){let e=[];for(let n=0;n<this.buffer.length;)e.push(this.childString(n)),n=this.buffer[n+3];return e.join(",")}childString(e){let n=this.buffer[e],r=this.buffer[e+3],i=this.set.types[n],s=i.name;if(/\W/.test(s)&&!i.isError&&(s=JSON.stringify(s)),e+=4,r==e)return s;let o=[];for(;e<r;)o.push(this.childString(e)),e=this.buffer[e+3];return s+"("+o.join(",")+")"}findChild(e,n,r,i,s){let{buffer:o}=this,l=-1;for(let a=e;a!=n&&!(_w(s,i,o[a+1],o[a+2])&&(l=a,r>0));a=o[a+3]);return l}slice(e,n,r){let i=this.buffer,s=new Uint16Array(n-e),o=0;for(let l=e,a=0;l<n;){s[a++]=i[l++],s[a++]=i[l++]-r;let c=s[a++]=i[l++]-r;s[a++]=i[l++]-e,o=Math.max(o,c)}return new pr(s,o,this.set)}}function _w(t,e,n,r){switch(t){case-2:return n<e;case-1:return r>=e&&n<e;case 0:return n<e&&r>e;case 1:return n<=e&&r>e;case 2:return r>e;case 4:return!0}}function po(t,e,n,r){for(var i;t.from==t.to||(n<1?t.from>=e:t.from>e)||(n>-1?t.to<=e:t.to<e);){let o=!r&&t instanceof qt&&t.index<0?null:t.parent;if(!o)return t;t=o}let s=r?0:Ae.IgnoreOverlays;if(r)for(let o=t,l=o.parent;l;o=l,l=o.parent)o instanceof qt&&o.index<0&&((i=l.enter(e,n,s))===null||i===void 0?void 0:i.from)!=o.from&&(t=l);for(;;){let o=t.enter(e,n,s);if(!o)return t;t=o}}class qw{cursor(e=0){return new Jh(this,e)}getChild(e,n=null,r=null){let i=cg(this,e,n,r);return i.length?i[0]:null}getChildren(e,n=null,r=null){return cg(this,e,n,r)}resolve(e,n=0){return po(this,e,n,!1)}resolveInner(e,n=0){return po(this,e,n,!0)}matchContext(e){return Xh(this.parent,e)}enterUnfinishedNodesBefore(e){let n=this.childBefore(e),r=this;for(;n;){let i=n.lastChild;if(!i||i.to!=n.to)break;i.type.isError&&i.from==i.to?(r=n,n=i.prevSibling):n=i}return r}get node(){return this}get next(){return this.parent}}class qt extends qw{constructor(e,n,r,i){super(),this._tree=e,this.from=n,this.index=r,this._parent=i}get type(){return this._tree.type}get name(){return this._tree.type.name}get to(){return this.from+this._tree.length}nextChild(e,n,r,i,s=0){for(let o=this;;){for(let{children:l,positions:a}=o._tree,c=n>0?l.length:-1;e!=c;e+=n){let u=l[e],h=a[e]+o.from,f;if(!(!(s&Ae.EnterBracketed&&u instanceof Se&&(f=zs.get(u))&&!f.overlay&&f.bracketed&&r>=h&&r<=h+u.length)&&!_w(i,r,h,h+u.length))){if(u instanceof pr){if(s&Ae.ExcludeBuffers)continue;let d=u.findChild(0,u.buffer.length,n,r-h,i);if(d>-1)return new Jn(new AM(o,u,e,h),null,d)}else if(s&Ae.IncludeAnonymous||!u.type.isAnonymous||bd(u)){let d;if(!(s&Ae.IgnoreMounts)&&(d=zs.get(u))&&!d.overlay)return new qt(d.tree,h,e,o);let p=new qt(u,h,e,o);return s&Ae.IncludeAnonymous||!p.type.isAnonymous?p:p.nextChild(n<0?u.children.length-1:0,n,r,i,s)}}}if(s&Ae.IncludeAnonymous||!o.type.isAnonymous||(o.index>=0?e=o.index+n:e=n<0?-1:o._parent._tree.children.length,o=o._parent,!o))return null}}get firstChild(){return this.nextChild(0,1,0,4)}get lastChild(){return this.nextChild(this._tree.children.length-1,-1,0,4)}childAfter(e){return this.nextChild(0,1,e,2)}childBefore(e){return this.nextChild(this._tree.children.length-1,-1,e,-2)}prop(e){return this._tree.prop(e)}enter(e,n,r=0){let i;if(!(r&Ae.IgnoreOverlays)&&(i=zs.get(this._tree))&&i.overlay){let s=e-this.from,o=r&Ae.EnterBracketed&&i.bracketed;for(let{from:l,to:a}of i.overlay)if((n>0||o?l<=s:l<s)&&(n<0||o?a>=s:a>s))return new qt(i.tree,i.overlay[0].from+this.from,-1,this)}return this.nextChild(0,1,e,n,r)}nextSignificantParent(){let e=this;for(;e.type.isAnonymous&&e._parent;)e=e._parent;return e}get parent(){return this._parent?this._parent.nextSignificantParent():null}get nextSibling(){return this._parent&&this.index>=0?this._parent.nextChild(this.index+1,1,0,4):null}get prevSibling(){return this._parent&&this.index>=0?this._parent.nextChild(this.index-1,-1,0,4):null}get tree(){return this._tree}toTree(){return this._tree}toString(){return this._tree.toString()}}function cg(t,e,n,r){let i=t.cursor(),s=[];if(!i.firstChild())return s;if(n!=null){for(let o=!1;!o;)if(o=i.type.is(n),!i.nextSibling())return s}for(;;){if(r!=null&&i.type.is(r))return s;if(i.type.is(e)&&s.push(i.node),!i.nextSibling())return r==null?s:[]}}function Xh(t,e,n=e.length-1){for(let r=t;n>=0;r=r.parent){if(!r)return!1;if(!r.type.isAnonymous){if(e[n]&&e[n]!=r.name)return!1;n--}}return!0}class AM{constructor(e,n,r,i){this.parent=e,this.buffer=n,this.index=r,this.start=i}}class Jn extends qw{get name(){return this.type.name}get from(){return this.context.start+this.context.buffer.buffer[this.index+1]}get to(){return this.context.start+this.context.buffer.buffer[this.index+2]}constructor(e,n,r){super(),this.context=e,this._parent=n,this.index=r,this.type=e.buffer.set.types[e.buffer.buffer[r]]}child(e,n,r){let{buffer:i}=this.context,s=i.findChild(this.index+4,i.buffer[this.index+3],e,n-this.context.start,r);return s<0?null:new Jn(this.context,this,s)}get firstChild(){return this.child(1,0,4)}get lastChild(){return this.child(-1,0,4)}childAfter(e){return this.child(1,e,2)}childBefore(e){return this.child(-1,e,-2)}prop(e){return this.type.prop(e)}enter(e,n,r=0){if(r&Ae.ExcludeBuffers)return null;let{buffer:i}=this.context,s=i.findChild(this.index+4,i.buffer[this.index+3],n>0?1:-1,e-this.context.start,n);return s<0?null:new Jn(this.context,this,s)}get parent(){return this._parent||this.context.parent.nextSignificantParent()}externalSibling(e){return this._parent?null:this.context.parent.nextChild(this.context.index+e,e,0,4)}get nextSibling(){let{buffer:e}=this.context,n=e.buffer[this.index+3];return n<(this._parent?e.buffer[this._parent.index+3]:e.buffer.length)?new Jn(this.context,this._parent,n):this.externalSibling(1)}get prevSibling(){let{buffer:e}=this.context,n=this._parent?this._parent.index+4:0;return this.index==n?this.externalSibling(-1):new Jn(this.context,this._parent,e.findChild(n,this.index,-1,0,4))}get tree(){return null}toTree(){let e=[],n=[],{buffer:r}=this.context,i=this.index+4,s=r.buffer[this.index+3];if(s>i){let o=r.buffer[this.index+1];e.push(r.slice(i,s,o)),n.push(0)}return new Se(this.type,e,n,this.to-this.from)}toString(){return this.context.buffer.childString(this.index)}}function Ww(t){if(!t.length)return null;let e=0,n=t[0];for(let s=1;s<t.length;s++){let o=t[s];(o.from>n.from||o.to<n.to)&&(n=o,e=s)}let r=n instanceof qt&&n.index<0?null:n.parent,i=t.slice();return r?i[e]=r:i.splice(e,1),new PM(i,n)}class PM{constructor(e,n){this.heads=e,this.node=n}get next(){return Ww(this.heads)}}function EM(t,e,n){let r=t.resolveInner(e,n),i=null;for(let s=r instanceof qt?r:r.context.parent;s;s=s.parent)if(s.index<0){let o=s.parent;(i||(i=[r])).push(o.resolve(e,n)),s=o}else{let o=zs.get(s.tree);if(o&&o.overlay&&o.overlay[0].from<=e&&o.overlay[o.overlay.length-1].to>=e){let l=new qt(o.tree,o.overlay[0].from+s.from,-1,s);(i||(i=[r])).push(po(l,e,n,!1))}}return i?Ww(i):r}class Jh{get name(){return this.type.name}constructor(e,n=0){if(this.buffer=null,this.stack=[],this.index=0,this.bufferNode=null,this.mode=n&~Ae.EnterBracketed,e instanceof qt)this.yieldNode(e);else{this._tree=e.context.parent,this.buffer=e.context;for(let r=e._parent;r;r=r._parent)this.stack.unshift(r.index);this.bufferNode=e,this.yieldBuf(e.index)}}yieldNode(e){return e?(this._tree=e,this.type=e.type,this.from=e.from,this.to=e.to,!0):!1}yieldBuf(e,n){this.index=e;let{start:r,buffer:i}=this.buffer;return this.type=n||i.set.types[i.buffer[e]],this.from=r+i.buffer[e+1],this.to=r+i.buffer[e+2],!0}yield(e){return e?e instanceof qt?(this.buffer=null,this.yieldNode(e)):(this.buffer=e.context,this.yieldBuf(e.index,e.type)):!1}toString(){return this.buffer?this.buffer.buffer.childString(this.index):this._tree.toString()}enterChild(e,n,r){if(!this.buffer)return this.yield(this._tree.nextChild(e<0?this._tree._tree.children.length-1:0,e,n,r,this.mode));let{buffer:i}=this.buffer,s=i.findChild(this.index+4,i.buffer[this.index+3],e,n-this.buffer.start,r);return s<0?!1:(this.stack.push(this.index),this.yieldBuf(s))}firstChild(){return this.enterChild(1,0,4)}lastChild(){return this.enterChild(-1,0,4)}childAfter(e){return this.enterChild(1,e,2)}childBefore(e){return this.enterChild(-1,e,-2)}enter(e,n,r=this.mode){return this.buffer?r&Ae.ExcludeBuffers?!1:this.enterChild(1,e,n):this.yield(this._tree.enter(e,n,r))}parent(){if(!this.buffer)return this.yieldNode(this.mode&Ae.IncludeAnonymous?this._tree._parent:this._tree.parent);if(this.stack.length)return this.yieldBuf(this.stack.pop());let e=this.mode&Ae.IncludeAnonymous?this.buffer.parent:this.buffer.parent.nextSignificantParent();return this.buffer=null,this.yieldNode(e)}sibling(e){if(!this.buffer)return this._tree._parent?this.yield(this._tree.index<0?null:this._tree._parent.nextChild(this._tree.index+e,e,0,4,this.mode)):!1;let{buffer:n}=this.buffer,r=this.stack.length-1;if(e<0){let i=r<0?0:this.stack[r]+4;if(this.index!=i)return this.yieldBuf(n.findChild(i,this.index,-1,0,4))}else{let i=n.buffer[this.index+3];if(i<(r<0?n.buffer.length:n.buffer[this.stack[r]+3]))return this.yieldBuf(i)}return r<0?this.yield(this.buffer.parent.nextChild(this.buffer.index+e,e,0,4,this.mode)):!1}nextSibling(){return this.sibling(1)}prevSibling(){return this.sibling(-1)}atLastNode(e){let n,r,{buffer:i}=this;if(i){if(e>0){if(this.index<i.buffer.buffer.length)return!1}else for(let s=0;s<this.index;s++)if(i.buffer.buffer[s+3]<this.index)return!1;({index:n,parent:r}=i)}else({index:n,_parent:r}=this._tree);for(;r;{index:n,_parent:r}=r)if(n>-1)for(let s=n+e,o=e<0?-1:r._tree.children.length;s!=o;s+=e){let l=r._tree.children[s];if(this.mode&Ae.IncludeAnonymous||l instanceof pr||!l.type.isAnonymous||bd(l))return!1}return!0}move(e,n){if(n&&this.enterChild(e,0,4))return!0;for(;;){if(this.sibling(e))return!0;if(this.atLastNode(e)||!this.parent())return!1}}next(e=!0){return this.move(1,e)}prev(e=!0){return this.move(-1,e)}moveTo(e,n=0){for(;(this.from==this.to||(n<1?this.from>=e:this.from>e)||(n>-1?this.to<=e:this.to<e))&&this.parent(););for(;this.enterChild(1,e,n););return this}get node(){if(!this.buffer)return this._tree;let e=this.bufferNode,n=null,r=0;if(e&&e.context==this.buffer)e:for(let i=this.index,s=this.stack.length;s>=0;){for(let o=e;o;o=o._parent)if(o.index==i){if(i==this.index)return o;n=o,r=s+1;break e}i=this.stack[--s]}for(let i=r;i<this.stack.length;i++)n=new Jn(this.buffer,n,this.stack[i]);return this.bufferNode=new Jn(this.buffer,n,this.index)}get tree(){return this.buffer?null:this._tree._tree}iterate(e,n){for(let r=0;;){let i=!1;if(this.type.isAnonymous||e(this)!==!1){if(this.firstChild()){r++;continue}this.type.isAnonymous||(i=!0)}for(;;){if(i&&n&&n(this),i=this.type.isAnonymous,!r)return;if(this.nextSibling())break;this.parent(),r--,i=!0}}}matchContext(e){if(!this.buffer)return Xh(this.node.parent,e);let{buffer:n}=this.buffer,{types:r}=n.set;for(let i=e.length-1,s=this.stack.length-1;i>=0;s--){if(s<0)return Xh(this._tree,e,i);let o=r[n.buffer[this.stack[s]]];if(!o.isAnonymous){if(e[i]&&e[i]!=o.name)return!1;i--}}return!0}}function bd(t){return t.children.some(e=>e instanceof pr||!e.type.isAnonymous||bd(e))}function LM(t){var e;let{buffer:n,nodeSet:r,maxBufferLength:i=RM,reused:s=[],minRepeatType:o=r.types.length}=t,l=Array.isArray(n)?new Cd(n,n.length):n,a=r.types,c=0,u=0;function h(k,C,b,I,W,N){let{id:q,start:j,end:M,size:$}=l,Q=u,de=c;if($<0)if(l.next(),$==-1){let H=s[q];b.push(H),I.push(j-k);return}else if($==-3){c=q;return}else if($==-4){u=q;return}else throw new RangeError(`Unrecognized record size: ${$}`);let ae=a[q],T,A,P=j-k;if(M-j<=i&&(A=w(l.pos-C,W))){let H=new Uint16Array(A.size-A.skip),G=l.pos-A.size,re=H.length;for(;l.pos>G;)re=g(A.start,H,re);T=new pr(H,M-A.start,r),P=A.start-k}else{let H=l.pos-$;l.next();let G=[],re=[],pe=q>=o?q:-1,ke=0,se=M;for(;l.pos>H;)pe>=0&&l.id==pe&&l.size>=0?(l.end<=se-i&&(p(G,re,j,ke,l.end,se,pe,Q,de),ke=G.length,se=l.end),l.next()):N>2500?f(j,H,G,re):h(j,H,G,re,pe,N+1);if(pe>=0&&ke>0&&ke<G.length&&p(G,re,j,ke,j,se,pe,Q,de),G.reverse(),re.reverse(),pe>-1&&ke>0){let je=d(ae,de);T=Td(ae,G,re,0,G.length,0,M-j,je,je)}else T=m(ae,G,re,M-j,Q-M,de)}b.push(T),I.push(P)}function f(k,C,b,I){let W=[],N=0,q=-1;for(;l.pos>C;){let{id:j,start:M,end:$,size:Q}=l;if(Q>4)l.next();else{if(q>-1&&M<q)break;q<0&&(q=$-i),W.push(j,M,$),N++,l.next()}}if(N){let j=new Uint16Array(N*4),M=W[W.length-2];for(let $=W.length-3,Q=0;$>=0;$-=3)j[Q++]=W[$],j[Q++]=W[$+1]-M,j[Q++]=W[$+2]-M,j[Q++]=Q;b.push(new pr(j,W[2]-M,r)),I.push(M-k)}}function d(k,C){return(b,I,W)=>{let N=0,q=b.length-1,j,M;if(q>=0&&(j=b[q])instanceof Se){if(!q&&j.type==k&&j.length==W)return j;(M=j.prop(Z.lookAhead))&&(N=I[q]+j.length+M)}return m(k,b,I,W,N,C)}}function p(k,C,b,I,W,N,q,j,M){let $=[],Q=[];for(;k.length>I;)$.push(k.pop()),Q.push(C.pop()+b-W);k.push(m(r.types[q],$,Q,N-W,j-N,M)),C.push(W-b)}function m(k,C,b,I,W,N,q){if(N){let j=[Z.contextHash,N];q=q?[j].concat(q):[j]}if(W>25){let j=[Z.lookAhead,W];q=q?[j].concat(q):[j]}return new Se(k,C,b,I,q)}function w(k,C){let b=l.fork(),I=0,W=0,N=0,q=b.end-i,j={size:0,start:0,skip:0};e:for(let M=b.pos-k;b.pos>M;){let $=b.size;if(b.id==C&&$>=0){j.size=I,j.start=W,j.skip=N,N+=4,I+=4,b.next();continue}let Q=b.pos-$;if($<0||Q<M||b.start<q)break;let de=b.id>=o?4:0,ae=b.start;for(b.next();b.pos>Q;){if(b.size<0)if(b.size==-3||b.size==-4)de+=4;else break e;else b.id>=o&&(de+=4);b.next()}W=ae,I+=$,N+=de}return(C<0||I==k)&&(j.size=I,j.start=W,j.skip=N),j.size>4?j:void 0}function g(k,C,b){let{id:I,start:W,end:N,size:q}=l;if(l.next(),q>=0&&I<o){let j=b;if(q>4){let M=l.pos-(q-4);for(;l.pos>M;)b=g(k,C,b)}C[--b]=j,C[--b]=N-k,C[--b]=W-k,C[--b]=I}else q==-3?c=I:q==-4&&(u=I);return b}let v=[],y=[];for(;l.pos>0;)h(t.start||0,t.bufferStart||0,v,y,-1,0);let x=(e=t.length)!==null&&e!==void 0?e:v.length?y[0]+v[0].length:0;return new Se(a[t.topID],v.reverse(),y.reverse(),x)}const ug=new WeakMap;function zl(t,e){if(!t.isAnonymous||e instanceof pr||e.type!=t)return 1;let n=ug.get(e);if(n==null){n=1;for(let r of e.children){if(r.type!=t||!(r instanceof Se)){n=1;break}n+=zl(t,r)}ug.set(e,n)}return n}function Td(t,e,n,r,i,s,o,l,a){let c=0;for(let p=r;p<i;p++)c+=zl(t,e[p]);let u=Math.ceil(c*1.5/8),h=[],f=[];function d(p,m,w,g,v){for(let y=w;y<g;){let x=y,k=m[y],C=zl(t,p[y]);for(y++;y<g;y++){let b=zl(t,p[y]);if(C+b>=u)break;C+=b}if(y==x+1){if(C>u){let b=p[x];d(b.children,b.positions,0,b.children.length,m[x]+v);continue}h.push(p[x])}else{let b=m[y-1]+p[y-1].length-k;h.push(Td(t,p,m,x,y,k,b,null,a))}f.push(k+v-s)}}return d(e,n,r,i,0),(l||a)(h,f,o)}class qr{constructor(e,n,r,i,s=!1,o=!1){this.from=e,this.to=n,this.tree=r,this.offset=i,this.open=(s?1:0)|(o?2:0)}get openStart(){return(this.open&1)>0}get openEnd(){return(this.open&2)>0}static addTree(e,n=[],r=!1){let i=[new qr(0,e.length,e,0,!1,r)];for(let s of n)s.to>e.length&&i.push(s);return i}static applyChanges(e,n,r=128){if(!n.length)return e;let i=[],s=1,o=e.length?e[0]:null;for(let l=0,a=0,c=0;;l++){let u=l<n.length?n[l]:null,h=u?u.fromA:1e9;if(h-a>=r)for(;o&&o.from<h;){let f=o;if(a>=f.from||h<=f.to||c){let d=Math.max(f.from,a)-c,p=Math.min(f.to,h)-c;f=d>=p?null:new qr(d,p,f.tree,f.offset+c,l>0,!!u)}if(f&&i.push(f),o.to>h)break;o=s<e.length?e[s++]:null}if(!u)break;a=u.toA,c=u.toA-u.toB}return i}}class Fw{startParse(e,n,r){return typeof e=="string"&&(e=new DM(e)),r=r?r.length?r.map(i=>new su(i.from,i.to)):[new su(0,0)]:[new su(0,e.length)],this.createParse(e,n||[],r)}parse(e,n,r){let i=this.startParse(e,n,r);for(;;){let s=i.advance();if(s)return s}}}class DM{constructor(e){this.string=e}get length(){return this.string.length}chunk(e){return this.string.slice(e)}get lineChunks(){return!1}read(e,n){return this.string.slice(e,n)}}new Z({perNode:!0});let OM=0;class Lt{constructor(e,n,r,i){this.name=e,this.set=n,this.base=r,this.modified=i,this.id=OM++}toString(){let{name:e}=this;for(let n of this.modified)n.name&&(e=`${n.name}(${e})`);return e}static define(e,n){let r=typeof e=="string"?e:"?";if(e instanceof Lt&&(n=e),n!=null&&n.base)throw new Error("Can not derive from a modified tag");let i=new Lt(r,[],null,[]);if(i.set.push(i),n)for(let s of n.set)i.set.push(s);return i}static defineModifier(e){let n=new ba(e);return r=>r.modified.indexOf(n)>-1?r:ba.get(r.base||r,r.modified.concat(n).sort((i,s)=>i.id-s.id))}}let NM=0;class ba{constructor(e){this.name=e,this.instances=[],this.id=NM++}static get(e,n){if(!n.length)return e;let r=n[0].instances.find(l=>l.base==e&&BM(n,l.modified));if(r)return r;let i=[],s=new Lt(e.name,i,e,n);for(let l of n)l.instances.push(s);let o=_M(n);for(let l of e.set)if(!l.modified.length)for(let a of o)i.push(ba.get(l,a));return s}}function BM(t,e){return t.length==e.length&&t.every((n,r)=>n==e[r])}function _M(t){let e=[[]];for(let n=0;n<t.length;n++)for(let r=0,i=e.length;r<i;r++)e.push(e[r].concat(t[n]));return e.sort((n,r)=>r.length-n.length)}function qM(t){let e=Object.create(null);for(let n in t){let r=t[n];Array.isArray(r)||(r=[r]);for(let i of n.split(" "))if(i){let s=[],o=2,l=i;for(let h=0;;){if(l=="..."&&h>0&&h+3==i.length){o=1;break}let f=/^"(?:[^"\\]|\\.)*?"|[^\/!]+/.exec(l);if(!f)throw new RangeError("Invalid path: "+i);if(s.push(f[0]=="*"?"":f[0][0]=='"'?JSON.parse(f[0]):f[0]),h+=f[0].length,h==i.length)break;let d=i[h++];if(h==i.length&&d=="!"){o=0;break}if(d!="/")throw new RangeError("Invalid path: "+i);l=i.slice(h)}let a=s.length-1,c=s[a];if(!c)throw new RangeError("Invalid path: "+i);let u=new mo(r,o,a>0?s.slice(0,a):null);e[c]=u.sort(e[c])}}return zw.add(e)}const zw=new Z({combine(t,e){let n,r,i;for(;t||e;){if(!t||e&&t.depth<=e.depth?(i=e,e=e.next):(i=t,t=t.next),n&&n.mode==i.mode&&!i.context&&!n.context)continue;let s=new mo(i.tags,i.mode,i.context);n?n.next=s:r=s,n=s}return r}});class mo{constructor(e,n,r,i){this.tags=e,this.mode=n,this.context=r,this.next=i}get opaque(){return this.mode==0}get inherit(){return this.mode==1}sort(e){return!e||e.depth<this.depth?(this.next=e,this):(e.next=this.sort(e.next),e)}get depth(){return this.context?this.context.length:0}}mo.empty=new mo([],2,null);function jw(t,e){let n=Object.create(null);for(let s of t)if(!Array.isArray(s.tag))n[s.tag.id]=s.class;else for(let o of s.tag)n[o.id]=s.class;let{scope:r,all:i=null}=e||{};return{style:s=>{let o=i;for(let l of s)for(let a of l.set){let c=n[a.id];if(c){o=o?o+" "+c:c;break}}return o},scope:r}}function WM(t,e){let n=null;for(let r of t){let i=r.style(e);i&&(n=n?n+" "+i:i)}return n}function FM(t,e,n,r=0,i=t.length){let s=new zM(r,Array.isArray(e)?e:[e],n);s.highlightRange(t.cursor(),r,i,"",s.highlighters),s.flush(i)}class zM{constructor(e,n,r){this.at=e,this.highlighters=n,this.span=r,this.class=""}startSpan(e,n){n!=this.class&&(this.flush(e),e>this.at&&(this.at=e),this.class=n)}flush(e){e>this.at&&this.class&&this.span(this.at,e,this.class)}highlightRange(e,n,r,i,s){let{type:o,from:l,to:a}=e;if(l>=r||a<=n)return;o.isTop&&(s=this.highlighters.filter(d=>!d.scope||d.scope(o)));let c=i,u=jM(e)||mo.empty,h=WM(s,u.tags);if(h&&(c&&(c+=" "),c+=h,u.mode==1&&(i+=(i?" ":"")+h)),this.startSpan(Math.max(n,l),c),u.opaque)return;let f=e.tree&&e.tree.prop(Z.mounted);if(f&&f.overlay){let d=e.node.enter(f.overlay[0].from+l,1),p=this.highlighters.filter(w=>!w.scope||w.scope(f.tree.type)),m=e.firstChild();for(let w=0,g=l;;w++){let v=w<f.overlay.length?f.overlay[w]:null,y=v?v.from+l:a,x=Math.max(n,g),k=Math.min(r,y);if(x<k&&m)for(;e.from<k&&(this.highlightRange(e,x,k,i,s),this.startSpan(Math.min(k,e.to),c),!(e.to>=y||!e.nextSibling())););if(!v||y>r)break;g=v.to+l,g>n&&(this.highlightRange(d.cursor(),Math.max(n,v.from+l),Math.min(r,g),"",p),this.startSpan(Math.min(r,g),c))}m&&e.parent()}else if(e.firstChild()){f&&(i="");do if(!(e.to<=n)){if(e.from>=r)break;this.highlightRange(e,n,r,i,s),this.startSpan(Math.min(r,e.to),c)}while(e.nextSibling());e.parent()}}}function jM(t){let e=t.type.prop(zw);for(;e&&e.context&&!t.matchContext(e.context);)e=e.next;return e||null}const B=Lt.define,dl=B(),jn=B(),hg=B(jn),fg=B(jn),Vn=B(),pl=B(Vn),ou=B(Vn),an=B(),Cr=B(an),on=B(),ln=B(),Zh=B(),ms=B(Zh),ml=B(),D={comment:dl,lineComment:B(dl),blockComment:B(dl),docComment:B(dl),name:jn,variableName:B(jn),typeName:hg,tagName:B(hg),propertyName:fg,attributeName:B(fg),className:B(jn),labelName:B(jn),namespace:B(jn),macroName:B(jn),literal:Vn,string:pl,docString:B(pl),character:B(pl),attributeValue:B(pl),number:ou,integer:B(ou),float:B(ou),bool:B(Vn),regexp:B(Vn),escape:B(Vn),color:B(Vn),url:B(Vn),keyword:on,self:B(on),null:B(on),atom:B(on),unit:B(on),modifier:B(on),operatorKeyword:B(on),controlKeyword:B(on),definitionKeyword:B(on),moduleKeyword:B(on),operator:ln,derefOperator:B(ln),arithmeticOperator:B(ln),logicOperator:B(ln),bitwiseOperator:B(ln),compareOperator:B(ln),updateOperator:B(ln),definitionOperator:B(ln),typeOperator:B(ln),controlOperator:B(ln),punctuation:Zh,separator:B(Zh),bracket:ms,angleBracket:B(ms),squareBracket:B(ms),paren:B(ms),brace:B(ms),content:an,heading:Cr,heading1:B(Cr),heading2:B(Cr),heading3:B(Cr),heading4:B(Cr),heading5:B(Cr),heading6:B(Cr),contentSeparator:B(an),list:B(an),quote:B(an),emphasis:B(an),strong:B(an),link:B(an),monospace:B(an),strikethrough:B(an),inserted:B(),deleted:B(),changed:B(),invalid:B(),meta:ml,documentMeta:B(ml),annotation:B(ml),processingInstruction:B(ml),definition:Lt.defineModifier("definition"),constant:Lt.defineModifier("constant"),function:Lt.defineModifier("function"),standard:Lt.defineModifier("standard"),local:Lt.defineModifier("local"),special:Lt.defineModifier("special")};for(let t in D){let e=D[t];e instanceof Lt&&(e.name=t)}jw([{tag:D.link,class:"tok-link"},{tag:D.heading,class:"tok-heading"},{tag:D.emphasis,class:"tok-emphasis"},{tag:D.strong,class:"tok-strong"},{tag:D.keyword,class:"tok-keyword"},{tag:D.atom,class:"tok-atom"},{tag:D.bool,class:"tok-bool"},{tag:D.url,class:"tok-url"},{tag:D.labelName,class:"tok-labelName"},{tag:D.inserted,class:"tok-inserted"},{tag:D.deleted,class:"tok-deleted"},{tag:D.literal,class:"tok-literal"},{tag:D.string,class:"tok-string"},{tag:D.number,class:"tok-number"},{tag:[D.regexp,D.escape,D.special(D.string)],class:"tok-string2"},{tag:D.variableName,class:"tok-variableName"},{tag:D.local(D.variableName),class:"tok-variableName tok-local"},{tag:D.definition(D.variableName),class:"tok-variableName tok-definition"},{tag:D.special(D.variableName),class:"tok-variableName2"},{tag:D.definition(D.propertyName),class:"tok-propertyName tok-definition"},{tag:D.typeName,class:"tok-typeName"},{tag:D.namespace,class:"tok-namespace"},{tag:D.className,class:"tok-className"},{tag:D.macroName,class:"tok-macroName"},{tag:D.propertyName,class:"tok-propertyName"},{tag:D.operator,class:"tok-operator"},{tag:D.comment,class:"tok-comment"},{tag:D.meta,class:"tok-meta"},{tag:D.invalid,class:"tok-invalid"},{tag:D.punctuation,class:"tok-punctuation"}]);function dg(t,e,n,r=0,i=0){e==null&&(e=t.search(/[^\s\u00a0]/),e==-1&&(e=t.length));let s=i;for(let o=r;o<e;o++)t.charCodeAt(o)==9?s+=n-s%n:s++;return s}class Vw{constructor(e,n,r,i){this.string=e,this.tabSize=n,this.indentUnit=r,this.overrideIndent=i,this.pos=0,this.start=0,this.lastColumnPos=0,this.lastColumnValue=0}eol(){return this.pos>=this.string.length}sol(){return this.pos==0}peek(){return this.string.charAt(this.pos)||void 0}next(){if(this.pos<this.string.length)return this.string.charAt(this.pos++)}eat(e){let n=this.string.charAt(this.pos),r;if(typeof e=="string"?r=n==e:r=n&&(e instanceof RegExp?e.test(n):e(n)),r)return++this.pos,n}eatWhile(e){let n=this.pos;for(;this.eat(e););return this.pos>n}eatSpace(){let e=this.pos;for(;/[\s\u00a0]/.test(this.string.charAt(this.pos));)++this.pos;return this.pos>e}skipToEnd(){this.pos=this.string.length}skipTo(e){let n=this.string.indexOf(e,this.pos);if(n>-1)return this.pos=n,!0}backUp(e){this.pos-=e}column(){return this.lastColumnPos<this.start&&(this.lastColumnValue=dg(this.string,this.start,this.tabSize,this.lastColumnPos,this.lastColumnValue),this.lastColumnPos=this.start),this.lastColumnValue}indentation(){var e;return(e=this.overrideIndent)!==null&&e!==void 0?e:dg(this.string,null,this.tabSize)}match(e,n,r){if(typeof e=="string"){let i=o=>r?o.toLowerCase():o,s=this.string.substr(this.pos,e.length);return i(s)==i(e)?(n!==!1&&(this.pos+=e.length),!0):null}else{let i=this.string.slice(this.pos).match(e);return i&&i.index>0?null:(i&&n!==!1&&(this.pos+=i[0].length),i)}}current(){return this.string.slice(this.start,this.pos)}}var lu;const Si=new Z;function VM(t){return z.define({combine:t?e=>e.concat(t):void 0})}const HM=new Z;class Kt{constructor(e,n,r=[],i=""){this.data=e,this.name=i,ee.prototype.hasOwnProperty("tree")||Object.defineProperty(ee.prototype,"tree",{get(){return et(this)}}),this.parser=n,this.extension=[mr.of(this),ee.languageData.of((s,o,l)=>{let a=pg(s,o,l),c=a.type.prop(Si);if(!c)return[];let u=s.facet(c),h=a.type.prop(HM);if(h){let f=a.resolve(o-a.from,l);for(let d of h)if(d.test(f,s)){let p=s.facet(d.facet);return d.type=="replace"?p:p.concat(u)}}return u})].concat(r)}isActiveAt(e,n,r=-1){return pg(e,n,r).type.prop(Si)==this.data}findRegions(e){let n=e.facet(mr);if((n==null?void 0:n.data)==this.data)return[{from:0,to:e.doc.length}];if(!n||!n.allowsNesting)return[];let r=[],i=(s,o)=>{if(s.prop(Si)==this.data){r.push({from:o,to:o+s.length});return}let l=s.prop(Z.mounted);if(l){if(l.tree.prop(Si)==this.data){if(l.overlay)for(let a of l.overlay)r.push({from:a.from+o,to:a.to+o});else r.push({from:o,to:o+s.length});return}else if(l.overlay){let a=r.length;if(i(l.tree,l.overlay[0].from+o),r.length>a)return}}for(let a=0;a<s.children.length;a++){let c=s.children[a];c instanceof Se&&i(c,s.positions[a]+o)}};return i(et(e),0),r}get allowsNesting(){return!0}}Kt.setState=K.define();function pg(t,e,n){let r=t.facet(mr),i=et(t).topNode;if(!r||r.allowsNesting)for(let s=i;s;s=s.enter(e,n,Ae.ExcludeBuffers|Ae.EnterBracketed))s.type.isTop&&(i=s);return i}function et(t){let e=t.field(Kt.state,!1);return e?e.tree:Se.empty}class UM{constructor(e){this.doc=e,this.cursorPos=0,this.string="",this.cursor=e.iter()}get length(){return this.doc.length}syncTo(e){return this.string=this.cursor.next(e-this.cursorPos).value,this.cursorPos=e+this.string.length,this.cursorPos-this.string.length}chunk(e){return this.syncTo(e),this.string}get lineChunks(){return!0}read(e,n){let r=this.cursorPos-this.string.length;return e<r||n>=this.cursorPos?this.doc.sliceString(e,n):this.string.slice(e-r,n-r)}}let gs=null;class Ki{constructor(e,n,r=[],i,s,o,l,a){this.parser=e,this.state=n,this.fragments=r,this.tree=i,this.treeLen=s,this.viewport=o,this.skipped=l,this.scheduleOn=a,this.parse=null,this.tempSkipped=[]}static create(e,n,r){return new Ki(e,n,[],Se.empty,0,r,[],null)}startParse(){return this.parser.startParse(new UM(this.state.doc),this.fragments)}work(e,n){return n!=null&&n>=this.state.doc.length&&(n=void 0),this.tree!=Se.empty&&this.isDone(n??this.state.doc.length)?(this.takeTree(),!0):this.withContext(()=>{var r;if(typeof e=="number"){let i=Date.now()+e;e=()=>Date.now()>i}for(this.parse||(this.parse=this.startParse()),n!=null&&(this.parse.stoppedAt==null||this.parse.stoppedAt>n)&&n<this.state.doc.length&&this.parse.stopAt(n);;){let i=this.parse.advance();if(i)if(this.fragments=this.withoutTempSkipped(qr.addTree(i,this.fragments,this.parse.stoppedAt!=null)),this.treeLen=(r=this.parse.stoppedAt)!==null&&r!==void 0?r:this.state.doc.length,this.tree=i,this.parse=null,this.treeLen<(n??this.state.doc.length))this.parse=this.startParse();else return!0;if(e())return!1}})}takeTree(){let e,n;this.parse&&(e=this.parse.parsedPos)>=this.treeLen&&((this.parse.stoppedAt==null||this.parse.stoppedAt>e)&&this.parse.stopAt(e),this.withContext(()=>{for(;!(n=this.parse.advance()););}),this.treeLen=e,this.tree=n,this.fragments=this.withoutTempSkipped(qr.addTree(this.tree,this.fragments,!0)),this.parse=null)}withContext(e){let n=gs;gs=this;try{return e()}finally{gs=n}}withoutTempSkipped(e){for(let n;n=this.tempSkipped.pop();)e=mg(e,n.from,n.to);return e}changes(e,n){let{fragments:r,tree:i,treeLen:s,viewport:o,skipped:l}=this;if(this.takeTree(),!e.empty){let a=[];if(e.iterChangedRanges((c,u,h,f)=>a.push({fromA:c,toA:u,fromB:h,toB:f})),r=qr.applyChanges(r,a),i=Se.empty,s=0,o={from:e.mapPos(o.from,-1),to:e.mapPos(o.to,1)},this.skipped.length){l=[];for(let c of this.skipped){let u=e.mapPos(c.from,1),h=e.mapPos(c.to,-1);u<h&&l.push({from:u,to:h})}}}return new Ki(this.parser,n,r,i,s,o,l,this.scheduleOn)}updateViewport(e){if(this.viewport.from==e.from&&this.viewport.to==e.to)return!1;this.viewport=e;let n=this.skipped.length;for(let r=0;r<this.skipped.length;r++){let{from:i,to:s}=this.skipped[r];i<e.to&&s>e.from&&(this.fragments=mg(this.fragments,i,s),this.skipped.splice(r--,1))}return this.skipped.length>=n?!1:(this.reset(),!0)}reset(){this.parse&&(this.takeTree(),this.parse=null)}skipUntilInView(e,n){this.skipped.push({from:e,to:n})}static getSkippingParser(e){return new class extends Fw{createParse(n,r,i){let s=i[0].from,o=i[i.length-1].to;return{parsedPos:s,advance(){let a=gs;if(a){for(let c of i)a.tempSkipped.push(c);e&&(a.scheduleOn=a.scheduleOn?Promise.all([a.scheduleOn,e]):e)}return this.parsedPos=o,new Se(St.none,[],[],o-s)},stoppedAt:null,stopAt(){}}}}}isDone(e){e=Math.min(e,this.state.doc.length);let n=this.fragments;return this.treeLen>=e&&n.length&&n[0].from==0&&n[0].to>=e}static get(){return gs}}function mg(t,e,n){return qr.applyChanges(t,[{fromA:e,toA:n,fromB:e,toB:n}])}class Qi{constructor(e){this.context=e,this.tree=e.tree}apply(e){if(!e.docChanged&&this.tree==this.context.tree)return this;let n=this.context.changes(e.changes,e.state),r=this.context.treeLen==e.startState.doc.length?void 0:Math.max(e.changes.mapPos(this.context.treeLen),n.viewport.to);return n.work(20,r)||n.takeTree(),new Qi(n)}static init(e){let n=Math.min(3e3,e.doc.length),r=Ki.create(e.facet(mr).parser,e,{from:0,to:n});return r.work(20,n)||r.takeTree(),new Qi(r)}}Kt.state=ze.define({create:Qi.init,update(t,e){for(let n of e.effects)if(n.is(Kt.setState))return n.value;return e.startState.facet(mr)!=e.state.facet(mr)?Qi.init(e.state):t.apply(e)}});let Hw=t=>{let e=setTimeout(()=>t(),500);return()=>clearTimeout(e)};typeof requestIdleCallback<"u"&&(Hw=t=>{let e=-1,n=setTimeout(()=>{e=requestIdleCallback(t,{timeout:400})},100);return()=>e<0?clearTimeout(n):cancelIdleCallback(e)});const au=typeof navigator<"u"&&(!((lu=navigator.scheduling)===null||lu===void 0)&&lu.isInputPending)?()=>navigator.scheduling.isInputPending():null,$M=Me.fromClass(class{constructor(e){this.view=e,this.working=null,this.workScheduled=0,this.chunkEnd=-1,this.chunkBudget=-1,this.work=this.work.bind(this),this.scheduleWork()}update(e){let n=this.view.state.field(Kt.state).context;(n.updateViewport(e.view.viewport)||this.view.viewport.to>n.treeLen)&&this.scheduleWork(),(e.docChanged||e.selectionSet)&&(this.view.hasFocus&&(this.chunkBudget+=50),this.scheduleWork()),this.checkAsyncSchedule(n)}scheduleWork(){if(this.working)return;let{state:e}=this.view,n=e.field(Kt.state);(n.tree!=n.context.tree||!n.context.isDone(e.doc.length))&&(this.working=Hw(this.work))}work(e){this.working=null;let n=Date.now();if(this.chunkEnd<n&&(this.chunkEnd<0||this.view.hasFocus)&&(this.chunkEnd=n+3e4,this.chunkBudget=3e3),this.chunkBudget<=0)return;let{state:r,viewport:{to:i}}=this.view,s=r.field(Kt.state);if(s.tree==s.context.tree&&s.context.isDone(i+1e5))return;let o=Date.now()+Math.min(this.chunkBudget,100,e&&!au?Math.max(25,e.timeRemaining()-5):1e9),l=s.context.treeLen<i&&r.doc.length>i+1e3,a=s.context.work(()=>au&&au()||Date.now()>o,i+(l?0:1e5));this.chunkBudget-=Date.now()-n,(a||this.chunkBudget<=0)&&(s.context.takeTree(),this.view.dispatch({effects:Kt.setState.of(new Qi(s.context))})),this.chunkBudget>0&&!(a&&!l)&&this.scheduleWork(),this.checkAsyncSchedule(s.context)}checkAsyncSchedule(e){e.scheduleOn&&(this.workScheduled++,e.scheduleOn.then(()=>this.scheduleWork()).catch(n=>st(this.view.state,n)).then(()=>this.workScheduled--),e.scheduleOn=null)}destroy(){this.working&&this.working()}isWorking(){return!!(this.working||this.workScheduled>0)}},{eventHandlers:{focus(){this.scheduleWork()}}}),mr=z.define({combine(t){return t.length?t[0]:null},enables:t=>[Kt.state,$M,F.contentAttributes.compute([t],e=>{let n=e.facet(t);return n&&n.name?{"data-language":n.name}:{}})]}),GM=z.define(),Rd=z.define({combine:t=>{if(!t.length)return"  ";let e=t[0];if(!e||/\S/.test(e)||Array.from(e).some(n=>n!=e[0]))throw new Error("Invalid indent unit: "+JSON.stringify(t[0]));return e}});function Kr(t){let e=t.facet(Rd);return e.charCodeAt(0)==9?t.tabSize*e.length:e.length}function go(t,e){let n="",r=t.tabSize,i=t.facet(Rd)[0];if(i=="	"){for(;e>=r;)n+="	",e-=r;i=" "}for(let s=0;s<e;s++)n+=i;return n}function Md(t,e){t instanceof ee&&(t=new lc(t));for(let r of t.state.facet(GM)){let i=r(t,e);if(i!==void 0)return i}let n=et(t.state);return n.length>=e?KM(t,n,e):null}class lc{constructor(e,n={}){this.state=e,this.options=n,this.unit=Kr(e)}lineAt(e,n=1){let r=this.state.doc.lineAt(e),{simulateBreak:i,simulateDoubleBreak:s}=this.options;return i!=null&&i>=r.from&&i<=r.to?s&&i==e?{text:"",from:e}:(n<0?i<e:i<=e)?{text:r.text.slice(i-r.from),from:i}:{text:r.text.slice(0,i-r.from),from:r.from}:r}textAfterPos(e,n=1){if(this.options.simulateDoubleBreak&&e==this.options.simulateBreak)return"";let{text:r,from:i}=this.lineAt(e,n);return r.slice(e-i,Math.min(r.length,e+100-i))}column(e,n=1){let{text:r,from:i}=this.lineAt(e,n),s=this.countColumn(r,e-i),o=this.options.overrideIndentation?this.options.overrideIndentation(i):-1;return o>-1&&(s+=o-this.countColumn(r,r.search(/\S|$/))),s}countColumn(e,n=e.length){return es(e,this.state.tabSize,n)}lineIndent(e,n=1){let{text:r,from:i}=this.lineAt(e,n),s=this.options.overrideIndentation;if(s){let o=s(i);if(o>-1)return o}return this.countColumn(r,r.search(/\S|$/))}get simulatedBreak(){return this.options.simulateBreak||null}}const Uw=new Z;function KM(t,e,n){let r=e.resolveStack(n),i=e.resolveInner(n,-1).resolve(n,0).enterUnfinishedNodesBefore(n);if(i!=r.node){let s=[];for(let o=i;o&&!(o.from<r.node.from||o.to>r.node.to||o.from==r.node.from&&o.type==r.node.type);o=o.parent)s.push(o);for(let o=s.length-1;o>=0;o--)r={node:s[o],next:r}}return $w(r,t,n)}function $w(t,e,n){for(let r=t;r;r=r.next){let i=YM(r.node);if(i)return i(Id.create(e,n,r))}return 0}function QM(t){return t.pos==t.options.simulateBreak&&t.options.simulateDoubleBreak}function YM(t){let e=t.type.prop(Uw);if(e)return e;let n=t.firstChild,r;if(n&&(r=n.type.prop(Z.closedBy))){let i=t.lastChild,s=i&&r.indexOf(i.name)>-1;return o=>e2(o,!0,1,void 0,s&&!QM(o)?i.from:void 0)}return t.parent==null?XM:null}function XM(){return 0}class Id extends lc{constructor(e,n,r){super(e.state,e.options),this.base=e,this.pos=n,this.context=r}get node(){return this.context.node}static create(e,n,r){return new Id(e,n,r)}get textAfter(){return this.textAfterPos(this.pos)}get baseIndent(){return this.baseIndentFor(this.node)}baseIndentFor(e){let n=this.state.doc.lineAt(e.from);for(;;){let r=e.resolve(n.from);for(;r.parent&&r.parent.from==r.from;)r=r.parent;if(JM(r,e))break;n=this.state.doc.lineAt(r.from)}return this.lineIndent(n.from)}continue(){return $w(this.context.next,this.base,this.pos)}}function JM(t,e){for(let n=e;n;n=n.parent)if(t==n)return!0;return!1}function ZM(t){let e=t.node,n=e.childAfter(e.from),r=e.lastChild;if(!n)return null;let i=t.options.simulateBreak,s=t.state.doc.lineAt(n.from),o=i==null||i<=s.from?s.to:Math.min(s.to,i);for(let l=n.to;;){let a=e.childAfter(l);if(!a||a==r)return null;if(!a.type.isSkipped){if(a.from>=o)return null;let c=/^ */.exec(s.text.slice(n.to-s.from))[0].length;return{from:n.from,to:n.to+c}}l=a.to}}function e2(t,e,n,r,i){let s=t.textAfter,o=s.match(/^\s*/)[0].length,l=r&&s.slice(o,o+r.length)==r||i==t.pos+o,a=ZM(t);return a?l?t.column(a.from):t.column(a.to):t.baseIndent+(l?0:t.unit*n)}const t2=200;function n2(){return ee.transactionFilter.of(t=>{if(!t.docChanged||!t.isUserEvent("input.type")&&!t.isUserEvent("input.complete"))return t;let e=t.startState.languageDataAt("indentOnInput",t.startState.selection.main.head);if(!e.length)return t;let n=t.newDoc,{head:r}=t.newSelection.main,i=n.lineAt(r);if(r>i.from+t2)return t;let s=n.sliceString(i.from,r);if(!e.some(c=>c.test(s)))return t;let{state:o}=t,l=-1,a=[];for(let{head:c}of o.selection.ranges){let u=o.doc.lineAt(c);if(u.from==l)continue;l=u.from;let h=Md(o,u.from);if(h==null)continue;let f=/^\s*/.exec(u.text)[0],d=go(o,h);f!=d&&a.push({from:u.from,to:u.from+f.length,insert:d})}return a.length?[t,{changes:a,sequential:!0}]:t})}const r2=z.define(),i2=new Z;function s2(t,e,n){let r=et(t);if(r.length<n)return null;let i=r.resolveStack(n,1),s=null;for(let o=i;o;o=o.next){let l=o.node;if(l.to<=n||l.from>n)continue;if(s&&l.from<e)break;let a=l.type.prop(i2);if(a&&(l.to<r.length-50||r.length==t.doc.length||!o2(l))){let c=a(l,t);c&&c.from<=n&&c.from>=e&&c.to>n&&(s=c)}}return s}function o2(t){let e=t.lastChild;return e&&e.to==t.to&&e.type.isError}function Ta(t,e,n){for(let r of t.facet(r2)){let i=r(t,e,n);if(i)return i}return s2(t,e,n)}function Gw(t,e){let n=e.mapPos(t.from,1),r=e.mapPos(t.to,-1);return n>=r?void 0:{from:n,to:r}}const ac=K.define({map:Gw}),Po=K.define({map:Gw});function Kw(t){let e=[];for(let{head:n}of t.state.selection.ranges)e.some(r=>r.from<=n&&r.to>=n)||e.push(t.lineBlockAt(n));return e}const Qr=ze.define({create(){return U.none},update(t,e){e.isUserEvent("delete")&&e.changes.iterChangedRanges((r,i)=>t=gg(t,r,i)),t=t.map(e.changes);let n=[];for(let r of e.effects)r.is(ac)&&!l2(t,r.value.from,r.value.to)?n.push(r.value):r.is(Po)&&(t=t.update({filter:(i,s)=>r.value.from!=i||r.value.to!=s,filterFrom:r.value.from,filterTo:r.value.to}));if(n.length){let{preparePlaceholder:r}=e.state.facet(Xw),i=n.map(s=>(r?U.replace({widget:new p2(r(e.state,s))}):vg).range(s.from,s.to));t=t.update({add:i})}return e.selection&&(t=gg(t,e.selection.main.head)),t},provide:t=>F.decorations.from(t),toJSON(t,e){let n=[];return t.between(0,e.doc.length,(r,i)=>{n.push(r,i)}),n},fromJSON(t){if(!Array.isArray(t)||t.length%2)throw new RangeError("Invalid JSON for fold state");let e=[];for(let n=0;n<t.length;){let r=t[n++],i=t[n++];if(typeof r!="number"||typeof i!="number")throw new RangeError("Invalid JSON for fold state");e.push(vg.range(r,i))}return U.set(e,!0)}});function gg(t,e,n=e){let r=!1;return t.between(e,n,(i,s)=>{i<n&&s>e&&(r=!0)}),r?t.update({filterFrom:e,filterTo:n,filter:(i,s)=>i>=n||s<=e}):t}function Ra(t,e,n){var r;let i=null;return(r=t.field(Qr,!1))===null||r===void 0||r.between(e,n,(s,o)=>{(!i||i.from>s)&&(i={from:s,to:o})}),i}function l2(t,e,n){let r=!1;return t.between(e,e,(i,s)=>{i==e&&s==n&&(r=!0)}),r}function Qw(t,e){return t.field(Qr,!1)?e:e.concat(K.appendConfig.of(Jw()))}const a2=t=>{for(let e of Kw(t)){let n=Ta(t.state,e.from,e.to);if(n)return t.dispatch({effects:Qw(t.state,[ac.of(n),Yw(t,n)])}),!0}return!1},c2=t=>{if(!t.state.field(Qr,!1))return!1;let e=[];for(let n of Kw(t)){let r=Ra(t.state,n.from,n.to);r&&e.push(Po.of(r),Yw(t,r,!1))}return e.length&&t.dispatch({effects:e}),e.length>0};function Yw(t,e,n=!0){let r=t.state.doc.lineAt(e.from).number,i=t.state.doc.lineAt(e.to).number;return F.announce.of(`${t.state.phrase(n?"Folded lines":"Unfolded lines")} ${r} ${t.state.phrase("to")} ${i}.`)}const u2=t=>{let{state:e}=t,n=[];for(let r=0;r<e.doc.length;){let i=t.lineBlockAt(r),s=Ta(e,i.from,i.to);s&&n.push(ac.of(s)),r=(s?t.lineBlockAt(s.to):i).to+1}return n.length&&t.dispatch({effects:Qw(t.state,n)}),!!n.length},h2=t=>{let e=t.state.field(Qr,!1);if(!e||!e.size)return!1;let n=[];return e.between(0,t.state.doc.length,(r,i)=>{n.push(Po.of({from:r,to:i}))}),t.dispatch({effects:n}),!0},f2=[{key:"Ctrl-Shift-[",mac:"Cmd-Alt-[",run:a2},{key:"Ctrl-Shift-]",mac:"Cmd-Alt-]",run:c2},{key:"Ctrl-Alt-[",run:u2},{key:"Ctrl-Alt-]",run:h2}],d2={placeholderDOM:null,preparePlaceholder:null,placeholderText:"…"},Xw=z.define({combine(t){return Zt(t,d2)}});function Jw(t){return[Qr,v2]}function Zw(t,e){let{state:n}=t,r=n.facet(Xw),i=o=>{let l=t.lineBlockAt(t.posAtDOM(o.target)),a=Ra(t.state,l.from,l.to);a&&t.dispatch({effects:Po.of(a)}),o.preventDefault()};if(r.placeholderDOM)return r.placeholderDOM(t,i,e);let s=document.createElement("span");return s.textContent=r.placeholderText,s.setAttribute("aria-label",n.phrase("folded code")),s.title=n.phrase("unfold"),s.className="cm-foldPlaceholder",s.onclick=i,s}const vg=U.replace({widget:new class extends qn{toDOM(t){return Zw(t,null)}}});class p2 extends qn{constructor(e){super(),this.value=e}eq(e){return this.value==e.value}toDOM(e){return Zw(e,this.value)}}const m2={openText:"⌄",closedText:"›",markerDOM:null,domEventHandlers:{},foldingChanged:()=>!1};class cu extends Cn{constructor(e,n){super(),this.config=e,this.open=n}eq(e){return this.config==e.config&&this.open==e.open}toDOM(e){if(this.config.markerDOM)return this.config.markerDOM(this.open);let n=document.createElement("span");return n.textContent=this.open?this.config.openText:this.config.closedText,n.title=e.state.phrase(this.open?"Fold line":"Unfold line"),n}}function g2(t={}){let e={...m2,...t},n=new cu(e,!0),r=new cu(e,!1),i=Me.fromClass(class{constructor(o){this.from=o.viewport.from,this.markers=this.buildMarkers(o)}update(o){(o.docChanged||o.viewportChanged||o.startState.facet(mr)!=o.state.facet(mr)||o.startState.field(Qr,!1)!=o.state.field(Qr,!1)||et(o.startState)!=et(o.state)||e.foldingChanged(o))&&(this.markers=this.buildMarkers(o.view))}buildMarkers(o){let l=new xn;for(let a of o.viewportLineBlocks){let c=Ra(o.state,a.from,a.to)?r:Ta(o.state,a.from,a.to)?n:null;c&&l.add(a.from,a.from,c)}return l.finish()}}),{domEventHandlers:s}=e;return[i,Ow({class:"cm-foldGutter",markers(o){var l;return((l=o.plugin(i))===null||l===void 0?void 0:l.markers)||X.empty},initialSpacer(){return new cu(e,!1)},domEventHandlers:{...s,click:(o,l,a)=>{if(s.click&&s.click(o,l,a))return!0;let c=Ra(o.state,l.from,l.to);if(c)return o.dispatch({effects:Po.of(c)}),!0;let u=Ta(o.state,l.from,l.to);return u?(o.dispatch({effects:ac.of(u)}),!0):!1}}}),Jw()]}const v2=F.baseTheme({".cm-foldPlaceholder":{backgroundColor:"#eee",border:"1px solid #ddd",color:"#888",borderRadius:".2em",margin:"0 1px",padding:"0 1px",cursor:"pointer"},".cm-foldGutter span":{padding:"0 1px",cursor:"pointer"}});class Eo{constructor(e,n){this.specs=e;let r;function i(l){let a=hr.newName();return(r||(r=Object.create(null)))["."+a]=l,a}const s=typeof n.all=="string"?n.all:n.all?i(n.all):void 0,o=n.scope;this.scope=o instanceof Kt?l=>l.prop(Si)==o.data:o?l=>l==o:void 0,this.style=jw(e.map(l=>({tag:l.tag,class:l.class||i(Object.assign({},l,{tag:null}))})),{all:s}).style,this.module=r?new hr(r):null,this.themeType=n.themeType}static define(e,n){return new Eo(e,n||{})}}const ef=z.define(),e1=z.define({combine(t){return t.length?[t[0]]:null}});function uu(t){let e=t.facet(ef);return e.length?e:t.facet(e1)}function t1(t,e){let n=[w2],r;return t instanceof Eo&&(t.module&&n.push(F.styleModule.of(t.module)),r=t.themeType),e!=null&&e.fallback?n.push(e1.of(t)):r?n.push(ef.computeN([F.darkTheme],i=>i.facet(F.darkTheme)==(r=="dark")?[t]:[])):n.push(ef.of(t)),n}class y2{constructor(e){this.markCache=Object.create(null),this.tree=et(e.state),this.decorations=this.buildDeco(e,uu(e.state)),this.decoratedTo=e.viewport.to}update(e){let n=et(e.state),r=uu(e.state),i=r!=uu(e.startState),{viewport:s}=e.view,o=e.changes.mapPos(this.decoratedTo,1);n.length<s.to&&!i&&n.type==this.tree.type&&o>=s.to?(this.decorations=this.decorations.map(e.changes),this.decoratedTo=o):(n!=this.tree||e.viewportChanged||i)&&(this.tree=n,this.decorations=this.buildDeco(e.view,r),this.decoratedTo=s.to)}buildDeco(e,n){if(!n||!this.tree.length)return U.none;let r=new xn;for(let{from:i,to:s}of e.visibleRanges)FM(this.tree,n,(o,l,a)=>{r.add(o,l,this.markCache[a]||(this.markCache[a]=U.mark({class:a})))},i,s);return r.finish()}}const w2=Zr.high(Me.fromClass(y2,{decorations:t=>t.decorations})),S2=Eo.define([{tag:D.meta,color:"#404740"},{tag:D.link,textDecoration:"underline"},{tag:D.heading,textDecoration:"underline",fontWeight:"bold"},{tag:D.emphasis,fontStyle:"italic"},{tag:D.strong,fontWeight:"bold"},{tag:D.strikethrough,textDecoration:"line-through"},{tag:D.keyword,color:"#708"},{tag:[D.atom,D.bool,D.url,D.contentSeparator,D.labelName],color:"#219"},{tag:[D.literal,D.inserted],color:"#164"},{tag:[D.string,D.deleted],color:"#a11"},{tag:[D.regexp,D.escape,D.special(D.string)],color:"#e40"},{tag:D.definition(D.variableName),color:"#00f"},{tag:D.local(D.variableName),color:"#30a"},{tag:[D.typeName,D.namespace],color:"#085"},{tag:D.className,color:"#167"},{tag:[D.special(D.variableName),D.macroName],color:"#256"},{tag:D.definition(D.propertyName),color:"#00c"},{tag:D.comment,color:"#940"},{tag:D.invalid,color:"#f00"}]),k2=F.baseTheme({"&.cm-focused .cm-matchingBracket":{backgroundColor:"#328c8252"},"&.cm-focused .cm-nonmatchingBracket":{backgroundColor:"#bb555544"}}),n1=1e4,r1="()[]{}",i1=z.define({combine(t){return Zt(t,{afterCursor:!0,brackets:r1,maxScanDistance:n1,renderMatch:b2})}}),x2=U.mark({class:"cm-matchingBracket"}),C2=U.mark({class:"cm-nonmatchingBracket"});function b2(t){let e=[],n=t.matched?x2:C2;return e.push(n.range(t.start.from,t.start.to)),t.end&&e.push(n.range(t.end.from,t.end.to)),e}function yg(t){let e=[],n=t.facet(i1);for(let r of t.selection.ranges){if(!r.empty)continue;let i=mn(t,r.head,-1,n)||r.head>0&&mn(t,r.head-1,1,n)||n.afterCursor&&(mn(t,r.head,1,n)||r.head<t.doc.length&&mn(t,r.head+1,-1,n));i&&(e=e.concat(n.renderMatch(i,t)))}return U.set(e,!0)}const T2=Me.fromClass(class{constructor(t){this.paused=!1,this.decorations=yg(t.state)}update(t){(t.docChanged||t.selectionSet||this.paused)&&(t.view.composing?(this.decorations=this.decorations.map(t.changes),this.paused=!0):(this.decorations=yg(t.state),this.paused=!1))}},{decorations:t=>t.decorations}),R2=[T2,k2];function M2(t={}){return[i1.of(t),R2]}const I2=new Z;function tf(t,e,n){let r=t.prop(e<0?Z.openedBy:Z.closedBy);if(r)return r;if(t.name.length==1){let i=n.indexOf(t.name);if(i>-1&&i%2==(e<0?1:0))return[n[i+e]]}return null}function nf(t){let e=t.type.prop(I2);return e?e(t.node):t}function mn(t,e,n,r={}){let i=r.maxScanDistance||n1,s=r.brackets||r1,o=et(t),l=o.resolveInner(e,n);for(let a=l;a;a=a.parent){let c=tf(a.type,n,s);if(c&&a.from<a.to){let u=nf(a);if(u&&(n>0?e>=u.from&&e<u.to:e>u.from&&e<=u.to))return A2(t,e,n,a,u,c,s)}}return P2(t,e,n,o,l.type,i,s)}function A2(t,e,n,r,i,s,o){let l=r.parent,a={from:i.from,to:i.to},c=0,u=l==null?void 0:l.cursor();if(u&&(n<0?u.childBefore(r.from):u.childAfter(r.to)))do if(n<0?u.to<=r.from:u.from>=r.to){if(c==0&&s.indexOf(u.type.name)>-1&&u.from<u.to){let h=nf(u);return{start:a,end:h?{from:h.from,to:h.to}:void 0,matched:!0}}else if(tf(u.type,n,o))c++;else if(tf(u.type,-n,o)){if(c==0){let h=nf(u);return{start:a,end:h&&h.from<h.to?{from:h.from,to:h.to}:void 0,matched:!1}}c--}}while(n<0?u.prevSibling():u.nextSibling());return{start:a,matched:!1}}function P2(t,e,n,r,i,s,o){if(n<0?!e:e==t.doc.length)return null;let l=n<0?t.sliceDoc(e-1,e):t.sliceDoc(e,e+1),a=o.indexOf(l);if(a<0||a%2==0!=n>0)return null;let c={from:n<0?e-1:e,to:n>0?e+1:e},u=t.doc.iterRange(e,n>0?t.doc.length:0),h=0;for(let f=0;!u.next().done&&f<=s;){let d=u.value;n<0&&(f+=d.length);let p=e+f*n;for(let m=n>0?0:d.length-1,w=n>0?d.length:-1;m!=w;m+=n){let g=o.indexOf(d[m]);if(!(g<0||r.resolveInner(p+m,1).type!=i))if(g%2==0==n>0)h++;else{if(h==1)return{start:c,end:{from:p+m,to:p+m+1},matched:g>>1==a>>1};h--}}n>0&&(f+=d.length)}return u.done?{start:c,matched:!1}:null}function E2(t){return{name:t.name||"",token:t.token,blankLine:t.blankLine||(()=>{}),startState:t.startState||(()=>!0),copyState:t.copyState||L2,indent:t.indent||(()=>null),languageData:t.languageData||{},tokenTable:t.tokenTable||Ed,mergeTokens:t.mergeTokens!==!1}}function L2(t){if(typeof t!="object")return t;let e={};for(let n in t){let r=t[n];e[n]=r instanceof Array?r.slice():r}return e}const wg=new WeakMap;class Ad extends Kt{constructor(e){let n=VM(e.languageData),r=E2(e),i,s=new class extends Fw{createParse(o,l,a){return new O2(i,o,l,a)}};super(n,s,[],e.name),this.topNode=_2(n,this),i=this,this.streamParser=r,this.stateAfter=new Z({perNode:!0}),this.tokenTable=e.tokenTable?new a1(r.tokenTable):B2}static define(e){return new Ad(e)}getIndent(e){let n,{overrideIndentation:r}=e.options;r&&(n=wg.get(e.state),n!=null&&n<e.pos-1e4&&(n=void 0));let i=Pd(this,e.node.tree,e.node.from,e.node.from,n??e.pos),s,o;if(i?(o=i.state,s=i.pos+1):(o=this.streamParser.startState(e.unit),s=e.node.from),e.pos-s>1e4)return null;for(;s<e.pos;){let a=e.state.doc.lineAt(s),c=Math.min(e.pos,a.to);if(a.length){let u=r?r(a.from):-1,h=new Vw(a.text,e.state.tabSize,e.unit,u<0?void 0:u);for(;h.pos<c-a.from;)o1(this.streamParser.token,h,o)}else this.streamParser.blankLine(o,e.unit);if(c==e.pos)break;s=a.to+1}let l=e.lineAt(e.pos);return r&&n==null&&wg.set(e.state,l.from),this.streamParser.indent(o,/^\s*(.*)/.exec(l.text)[1],e)}get allowsNesting(){return!1}}function Pd(t,e,n,r,i){let s=n>=r&&n+e.length<=i&&e.prop(t.stateAfter);if(s)return{state:t.streamParser.copyState(s),pos:n+e.length};for(let o=e.children.length-1;o>=0;o--){let l=e.children[o],a=n+e.positions[o],c=l instanceof Se&&a<i&&Pd(t,l,a,r,i);if(c)return c}return null}function s1(t,e,n,r,i){if(i&&n<=0&&r>=e.length)return e;!i&&n==0&&e.type==t.topNode&&(i=!0);for(let s=e.children.length-1;s>=0;s--){let o=e.positions[s],l=e.children[s],a;if(o<r&&l instanceof Se){if(!(a=s1(t,l,n-o,r-o,i)))break;return i?new Se(e.type,e.children.slice(0,s).concat(a),e.positions.slice(0,s+1),o+a.length):a}}return null}function D2(t,e,n,r,i){for(let s of e){let o=s.from+(s.openStart?25:0),l=s.to-(s.openEnd?25:0),a=o<=n&&l>n&&Pd(t,s.tree,0-s.offset,n,l),c;if(a&&a.pos<=r&&(c=s1(t,s.tree,n+s.offset,a.pos+s.offset,!1)))return{state:a.state,tree:c}}return{state:t.streamParser.startState(i?Kr(i):4),tree:Se.empty}}class O2{constructor(e,n,r,i){this.lang=e,this.input=n,this.fragments=r,this.ranges=i,this.stoppedAt=null,this.chunks=[],this.chunkPos=[],this.chunk=[],this.chunkReused=void 0,this.rangeIndex=0,this.to=i[i.length-1].to;let s=Ki.get(),o=i[0].from,{state:l,tree:a}=D2(e,r,o,this.to,s==null?void 0:s.state);this.state=l,this.parsedPos=this.chunkStart=o+a.length;for(let c=0;c<a.children.length;c++)this.chunks.push(a.children[c]),this.chunkPos.push(a.positions[c]);s&&this.parsedPos<s.viewport.from-1e5&&i.some(c=>c.from<=s.viewport.from&&c.to>=s.viewport.from)&&(this.state=this.lang.streamParser.startState(Kr(s.state)),s.skipUntilInView(this.parsedPos,s.viewport.from),this.parsedPos=s.viewport.from),this.moveRangeIndex()}advance(){let e=Ki.get(),n=this.stoppedAt==null?this.to:Math.min(this.to,this.stoppedAt),r=Math.min(n,this.chunkStart+512);for(e&&(r=Math.min(r,e.viewport.to));this.parsedPos<r;)this.parseLine(e);return this.chunkStart<this.parsedPos&&this.finishChunk(),this.parsedPos>=n?this.finish():e&&this.parsedPos>=e.viewport.to?(e.skipUntilInView(this.parsedPos,n),this.finish()):null}stopAt(e){this.stoppedAt=e}lineAfter(e){let n=this.input.chunk(e);if(this.input.lineChunks)n==`
`&&(n="");else{let r=n.indexOf(`
`);r>-1&&(n=n.slice(0,r))}return e+n.length<=this.to?n:n.slice(0,this.to-e)}nextLine(){let e=this.parsedPos,n=this.lineAfter(e),r=e+n.length;for(let i=this.rangeIndex;;){let s=this.ranges[i].to;if(s>=r||(n=n.slice(0,s-(r-n.length)),i++,i==this.ranges.length))break;let o=this.ranges[i].from,l=this.lineAfter(o);n+=l,r=o+l.length}return{line:n,end:r}}skipGapsTo(e,n,r){for(;;){let i=this.ranges[this.rangeIndex].to,s=e+n;if(r>0?i>s:i>=s)break;let o=this.ranges[++this.rangeIndex].from;n+=o-i}return n}moveRangeIndex(){for(;this.ranges[this.rangeIndex].to<this.parsedPos;)this.rangeIndex++}emitToken(e,n,r,i){let s=4;if(this.ranges.length>1){i=this.skipGapsTo(n,i,1),n+=i;let l=this.chunk.length;i=this.skipGapsTo(r,i,-1),r+=i,s+=this.chunk.length-l}let o=this.chunk.length-4;return this.lang.streamParser.mergeTokens&&s==4&&o>=0&&this.chunk[o]==e&&this.chunk[o+2]==n?this.chunk[o+2]=r:this.chunk.push(e,n,r,s),i}parseLine(e){let{line:n,end:r}=this.nextLine(),i=0,{streamParser:s}=this.lang,o=new Vw(n,e?e.state.tabSize:4,e?Kr(e.state):2);if(o.eol())s.blankLine(this.state,o.indentUnit);else for(;!o.eol();){let l=o1(s.token,o,this.state);if(l&&(i=this.emitToken(this.lang.tokenTable.resolve(l),this.parsedPos+o.start,this.parsedPos+o.pos,i)),o.start>1e4)break}this.parsedPos=r,this.moveRangeIndex(),this.parsedPos<this.to&&this.parsedPos++}finishChunk(){let e=Se.build({buffer:this.chunk,start:this.chunkStart,length:this.parsedPos-this.chunkStart,nodeSet:N2,topID:0,maxBufferLength:512,reused:this.chunkReused});e=new Se(e.type,e.children,e.positions,e.length,[[this.lang.stateAfter,this.lang.streamParser.copyState(this.state)]]),this.chunks.push(e),this.chunkPos.push(this.chunkStart-this.ranges[0].from),this.chunk=[],this.chunkReused=void 0,this.chunkStart=this.parsedPos}finish(){return new Se(this.lang.topNode,this.chunks,this.chunkPos,this.parsedPos-this.ranges[0].from).balance()}}function o1(t,e,n){e.start=e.pos;for(let r=0;r<10;r++){let i=t(e,n);if(e.pos>e.start)return i}throw new Error("Stream parser failed to advance stream.")}const Ed=Object.create(null),vo=[St.none],N2=new xd(vo),Sg=[],kg=Object.create(null),l1=Object.create(null);for(let[t,e]of[["variable","variableName"],["variable-2","variableName.special"],["string-2","string.special"],["def","variableName.definition"],["tag","tagName"],["attribute","attributeName"],["type","typeName"],["builtin","variableName.standard"],["qualifier","modifier"],["error","invalid"],["header","heading"],["property","propertyName"]])l1[t]=c1(Ed,e);class a1{constructor(e){this.extra=e,this.table=Object.assign(Object.create(null),l1)}resolve(e){return e?this.table[e]||(this.table[e]=c1(this.extra,e)):0}}const B2=new a1(Ed);function hu(t,e){Sg.indexOf(t)>-1||(Sg.push(t),console.warn(e))}function c1(t,e){let n=[];for(let l of e.split(" ")){let a=[];for(let c of l.split(".")){let u=t[c]||D[c];u?typeof u=="function"?a.length?a=a.map(u):hu(c,`Modifier ${c} used at start of tag`):a.length?hu(c,`Tag ${c} used as modifier`):a=Array.isArray(u)?u:[u]:hu(c,`Unknown highlighting tag ${c}`)}for(let c of a)n.push(c)}if(!n.length)return 0;let r=e.replace(/ /g,"_"),i=r+" "+n.map(l=>l.id),s=kg[i];if(s)return s.id;let o=kg[i]=St.define({id:vo.length,name:r,props:[qM({[r]:n})]});return vo.push(o),o.id}function _2(t,e){let n=St.define({id:vo.length,name:"Document",props:[Si.add(()=>t),Uw.add(()=>r=>e.getIndent(r))],top:!0});return vo.push(n),n}ue.RTL,ue.LTR;const q2=t=>{let{state:e}=t,n=e.doc.lineAt(e.selection.main.from),r=Dd(t.state,n.from);return r.line?W2(t):r.block?z2(t):!1};function Ld(t,e){return({state:n,dispatch:r})=>{if(n.readOnly)return!1;let i=t(e,n);return i?(r(n.update(i)),!0):!1}}const W2=Ld(H2,0),F2=Ld(u1,0),z2=Ld((t,e)=>u1(t,e,V2(e)),0);function Dd(t,e){let n=t.languageDataAt("commentTokens",e,1);return n.length?n[0]:{}}const vs=50;function j2(t,{open:e,close:n},r,i){let s=t.sliceDoc(r-vs,r),o=t.sliceDoc(i,i+vs),l=/\s*$/.exec(s)[0].length,a=/^\s*/.exec(o)[0].length,c=s.length-l;if(s.slice(c-e.length,c)==e&&o.slice(a,a+n.length)==n)return{open:{pos:r-l,margin:l&&1},close:{pos:i+a,margin:a&&1}};let u,h;i-r<=2*vs?u=h=t.sliceDoc(r,i):(u=t.sliceDoc(r,r+vs),h=t.sliceDoc(i-vs,i));let f=/^\s*/.exec(u)[0].length,d=/\s*$/.exec(h)[0].length,p=h.length-d-n.length;return u.slice(f,f+e.length)==e&&h.slice(p,p+n.length)==n?{open:{pos:r+f+e.length,margin:/\s/.test(u.charAt(f+e.length))?1:0},close:{pos:i-d-n.length,margin:/\s/.test(h.charAt(p-1))?1:0}}:null}function V2(t){let e=[];for(let n of t.selection.ranges){let r=t.doc.lineAt(n.from),i=n.to<=r.to?r:t.doc.lineAt(n.to);i.from>r.from&&i.from==n.to&&(i=n.to==r.to+1?r:t.doc.lineAt(n.to-1));let s=e.length-1;s>=0&&e[s].to>r.from?e[s].to=i.to:e.push({from:r.from+/^\s*/.exec(r.text)[0].length,to:i.to})}return e}function u1(t,e,n=e.selection.ranges){let r=n.map(s=>Dd(e,s.from).block);if(!r.every(s=>s))return null;let i=n.map((s,o)=>j2(e,r[o],s.from,s.to));if(t!=2&&!i.every(s=>s))return{changes:e.changes(n.map((s,o)=>i[o]?[]:[{from:s.from,insert:r[o].open+" "},{from:s.to,insert:" "+r[o].close}]))};if(t!=1&&i.some(s=>s)){let s=[];for(let o=0,l;o<i.length;o++)if(l=i[o]){let a=r[o],{open:c,close:u}=l;s.push({from:c.pos-a.open.length,to:c.pos+c.margin},{from:u.pos-u.margin,to:u.pos+a.close.length})}return{changes:s}}return null}function H2(t,e,n=e.selection.ranges){let r=[],i=-1;e:for(let{from:s,to:o}of n){let l=r.length,a=1e9,c;for(let u=s;u<=o;){let h=e.doc.lineAt(u);if(c==null&&(c=Dd(e,h.from).line,!c))continue e;if(h.from>i&&(s==o||o>h.from)){i=h.from;let f=/^\s*/.exec(h.text)[0].length,d=f==h.length,p=h.text.slice(f,f+c.length)==c?f:-1;f<h.text.length&&f<a&&(a=f),r.push({line:h,comment:p,token:c,indent:f,empty:d,single:!1})}u=h.to+1}if(a<1e9)for(let u=l;u<r.length;u++)r[u].indent<r[u].line.text.length&&(r[u].indent=a);r.length==l+1&&(r[l].single=!0)}if(t!=2&&r.some(s=>s.comment<0&&(!s.empty||s.single))){let s=[];for(let{line:l,token:a,indent:c,empty:u,single:h}of r)(h||!u)&&s.push({from:l.from+c,insert:a+" "});let o=e.changes(s);return{changes:o,selection:e.selection.map(o,1)}}else if(t!=1&&r.some(s=>s.comment>=0)){let s=[];for(let{line:o,comment:l,token:a}of r)if(l>=0){let c=o.from+l,u=c+a.length;o.text[u-o.from]==" "&&u++,s.push({from:c,to:u})}return{changes:s}}return null}const rf=_n.define(),U2=_n.define(),$2=z.define(),h1=z.define({combine(t){return Zt(t,{minDepth:100,newGroupDelay:500,joinToEvent:(e,n)=>n},{minDepth:Math.max,newGroupDelay:Math.min,joinToEvent:(e,n)=>(r,i)=>e(r,i)||n(r,i)})}}),f1=ze.define({create(){return gn.empty},update(t,e){let n=e.state.facet(h1),r=e.annotation(rf);if(r){let a=gt.fromTransaction(e,r.selection),c=r.side,u=c==0?t.undone:t.done;return a?u=Ma(u,u.length,n.minDepth,a):u=m1(u,e.startState.selection),new gn(c==0?r.rest:u,c==0?u:r.rest)}let i=e.annotation(U2);if((i=="full"||i=="before")&&(t=t.isolate()),e.annotation(Ne.addToHistory)===!1)return e.changes.empty?t:t.addMapping(e.changes.desc);let s=gt.fromTransaction(e),o=e.annotation(Ne.time),l=e.annotation(Ne.userEvent);return s?t=t.addChanges(s,o,l,n,e):e.selection&&(t=t.addSelection(e.startState.selection,o,l,n.newGroupDelay)),(i=="full"||i=="after")&&(t=t.isolate()),t},toJSON(t){return{done:t.done.map(e=>e.toJSON()),undone:t.undone.map(e=>e.toJSON())}},fromJSON(t){return new gn(t.done.map(gt.fromJSON),t.undone.map(gt.fromJSON))}});function G2(t={}){return[f1,h1.of(t),F.domEventHandlers({beforeinput(e,n){let r=e.inputType=="historyUndo"?d1:e.inputType=="historyRedo"?sf:null;return r?(e.preventDefault(),r(n)):!1}})]}function cc(t,e){return function({state:n,dispatch:r}){if(!e&&n.readOnly)return!1;let i=n.field(f1,!1);if(!i)return!1;let s=i.pop(t,n,e);return s?(r(s),!0):!1}}const d1=cc(0,!1),sf=cc(1,!1),K2=cc(0,!0),Q2=cc(1,!0);class gt{constructor(e,n,r,i,s){this.changes=e,this.effects=n,this.mapped=r,this.startSelection=i,this.selectionsAfter=s}setSelAfter(e){return new gt(this.changes,this.effects,this.mapped,this.startSelection,e)}toJSON(){var e,n,r;return{changes:(e=this.changes)===null||e===void 0?void 0:e.toJSON(),mapped:(n=this.mapped)===null||n===void 0?void 0:n.toJSON(),startSelection:(r=this.startSelection)===null||r===void 0?void 0:r.toJSON(),selectionsAfter:this.selectionsAfter.map(i=>i.toJSON())}}static fromJSON(e){return new gt(e.changes&&De.fromJSON(e.changes),[],e.mapped&&kn.fromJSON(e.mapped),e.startSelection&&R.fromJSON(e.startSelection),e.selectionsAfter.map(R.fromJSON))}static fromTransaction(e,n){let r=Nt;for(let i of e.startState.facet($2)){let s=i(e);s.length&&(r=r.concat(s))}return!r.length&&e.changes.empty?null:new gt(e.changes.invert(e.startState.doc),r,void 0,n||e.startState.selection,Nt)}static selection(e){return new gt(void 0,Nt,void 0,void 0,e)}}function Ma(t,e,n,r){let i=e+1>n+20?e-n-1:0,s=t.slice(i,e);return s.push(r),s}function Y2(t,e){let n=[],r=!1;return t.iterChangedRanges((i,s)=>n.push(i,s)),e.iterChangedRanges((i,s,o,l)=>{for(let a=0;a<n.length;){let c=n[a++],u=n[a++];l>=c&&o<=u&&(r=!0)}}),r}function X2(t,e){return t.ranges.length==e.ranges.length&&t.ranges.filter((n,r)=>n.empty!=e.ranges[r].empty).length===0}function p1(t,e){return t.length?e.length?t.concat(e):t:e}const Nt=[],J2=200;function m1(t,e){if(t.length){let n=t[t.length-1],r=n.selectionsAfter.slice(Math.max(0,n.selectionsAfter.length-J2));return r.length&&r[r.length-1].eq(e)?t:(r.push(e),Ma(t,t.length-1,1e9,n.setSelAfter(r)))}else return[gt.selection([e])]}function Z2(t){let e=t[t.length-1],n=t.slice();return n[t.length-1]=e.setSelAfter(e.selectionsAfter.slice(0,e.selectionsAfter.length-1)),n}function fu(t,e){if(!t.length)return t;let n=t.length,r=Nt;for(;n;){let i=eI(t[n-1],e,r);if(i.changes&&!i.changes.empty||i.effects.length){let s=t.slice(0,n);return s[n-1]=i,s}else e=i.mapped,n--,r=i.selectionsAfter}return r.length?[gt.selection(r)]:Nt}function eI(t,e,n){let r=p1(t.selectionsAfter.length?t.selectionsAfter.map(l=>l.map(e)):Nt,n);if(!t.changes)return gt.selection(r);let i=t.changes.map(e),s=e.mapDesc(t.changes,!0),o=t.mapped?t.mapped.composeDesc(s):s;return new gt(i,K.mapEffects(t.effects,e),o,t.startSelection.map(s),r)}const tI=/^(input\.type|delete)($|\.)/;class gn{constructor(e,n,r=0,i=void 0){this.done=e,this.undone=n,this.prevTime=r,this.prevUserEvent=i}isolate(){return this.prevTime?new gn(this.done,this.undone):this}addChanges(e,n,r,i,s){let o=this.done,l=o[o.length-1];return l&&l.changes&&!l.changes.empty&&e.changes&&(!r||tI.test(r))&&(!l.selectionsAfter.length&&n-this.prevTime<i.newGroupDelay&&i.joinToEvent(s,Y2(l.changes,e.changes))||r=="input.type.compose")?o=Ma(o,o.length-1,i.minDepth,new gt(e.changes.compose(l.changes),p1(K.mapEffects(e.effects,l.changes),l.effects),l.mapped,l.startSelection,Nt)):o=Ma(o,o.length,i.minDepth,e),new gn(o,Nt,n,r)}addSelection(e,n,r,i){let s=this.done.length?this.done[this.done.length-1].selectionsAfter:Nt;return s.length>0&&n-this.prevTime<i&&r==this.prevUserEvent&&r&&/^select($|\.)/.test(r)&&X2(s[s.length-1],e)?this:new gn(m1(this.done,e),this.undone,n,r)}addMapping(e){return new gn(fu(this.done,e),fu(this.undone,e),this.prevTime,this.prevUserEvent)}pop(e,n,r){let i=e==0?this.done:this.undone;if(i.length==0)return null;let s=i[i.length-1],o=s.selectionsAfter[0]||(s.startSelection?s.startSelection.map(s.changes.invertedDesc,1):n.selection);if(r&&s.selectionsAfter.length)return n.update({selection:s.selectionsAfter[s.selectionsAfter.length-1],annotations:rf.of({side:e,rest:Z2(i),selection:o}),userEvent:e==0?"select.undo":"select.redo",scrollIntoView:!0});if(s.changes){let l=i.length==1?Nt:i.slice(0,i.length-1);return s.mapped&&(l=fu(l,s.mapped)),n.update({changes:s.changes,selection:s.startSelection,effects:s.effects,annotations:rf.of({side:e,rest:l,selection:o}),filter:!1,userEvent:e==0?"undo":"redo",scrollIntoView:!0})}else return null}}gn.empty=new gn(Nt,Nt);const nI=[{key:"Mod-z",run:d1,preventDefault:!0},{key:"Mod-y",mac:"Mod-Shift-z",run:sf,preventDefault:!0},{linux:"Ctrl-Shift-z",run:sf,preventDefault:!0},{key:"Mod-u",run:K2,preventDefault:!0},{key:"Alt-u",mac:"Mod-Shift-u",run:Q2,preventDefault:!0}];function ts(t,e){return R.create(t.ranges.map(e),t.mainIndex)}function en(t,e){return t.update({selection:e,scrollIntoView:!0,userEvent:"select"})}function tn({state:t,dispatch:e},n){let r=ts(t.selection,n);return r.eq(t.selection,!0)?!1:(e(en(t,r)),!0)}function uc(t,e){return R.cursor(e?t.to:t.from)}function g1(t,e){return tn(t,n=>n.empty?t.moveByChar(n,e):uc(n,e))}function tt(t){return t.textDirectionAt(t.state.selection.main.head)==ue.LTR}const v1=t=>g1(t,!tt(t)),y1=t=>g1(t,tt(t));function w1(t,e){return tn(t,n=>n.empty?t.moveByGroup(n,e):uc(n,e))}const rI=t=>w1(t,!tt(t)),iI=t=>w1(t,tt(t));function sI(t,e,n){if(e.type.prop(n))return!0;let r=e.to-e.from;return r&&(r>2||/[^\s,.;:]/.test(t.sliceDoc(e.from,e.to)))||e.firstChild}function hc(t,e,n){let r=et(t).resolveInner(e.head),i=n?Z.closedBy:Z.openedBy;for(let a=e.head;;){let c=n?r.childAfter(a):r.childBefore(a);if(!c)break;sI(t,c,i)?r=c:a=n?c.to:c.from}let s=r.type.prop(i),o,l;return s&&(o=n?mn(t,r.from,1):mn(t,r.to,-1))&&o.matched?l=n?o.end.to:o.end.from:l=n?r.to:r.from,R.cursor(l,n?-1:1)}const oI=t=>tn(t,e=>hc(t.state,e,!tt(t))),lI=t=>tn(t,e=>hc(t.state,e,tt(t)));function S1(t,e){return tn(t,n=>{if(!n.empty)return uc(n,e);let r=t.moveVertically(n,e);return r.head!=n.head?r:t.moveToLineBoundary(n,e)})}const k1=t=>S1(t,!1),x1=t=>S1(t,!0);function C1(t){let e=t.scrollDOM.clientHeight<t.scrollDOM.scrollHeight-2,n=0,r=0,i;if(e){for(let s of t.state.facet(F.scrollMargins)){let o=s(t);o!=null&&o.top&&(n=Math.max(o==null?void 0:o.top,n)),o!=null&&o.bottom&&(r=Math.max(o==null?void 0:o.bottom,r))}i=t.scrollDOM.clientHeight-n-r}else i=(t.dom.ownerDocument.defaultView||window).innerHeight;return{marginTop:n,marginBottom:r,selfScroll:e,height:Math.max(t.defaultLineHeight,i-5)}}function b1(t,e){let n=C1(t),{state:r}=t,i=ts(r.selection,o=>o.empty?t.moveVertically(o,e,n.height):uc(o,e));if(i.eq(r.selection))return!1;let s;if(n.selfScroll){let o=t.coordsAtPos(r.selection.main.head),l=t.scrollDOM.getBoundingClientRect(),a=l.top+n.marginTop,c=l.bottom-n.marginBottom;o&&o.top>a&&o.bottom<c&&(s=F.scrollIntoView(i.main.head,{y:"start",yMargin:o.top-a}))}return t.dispatch(en(r,i),{effects:s}),!0}const xg=t=>b1(t,!1),of=t=>b1(t,!0);function Sr(t,e,n){let r=t.lineBlockAt(e.head),i=t.moveToLineBoundary(e,n);if(i.head==e.head&&i.head!=(n?r.to:r.from)&&(i=t.moveToLineBoundary(e,n,!1)),!n&&i.head==r.from&&r.length){let s=/^\s*/.exec(t.state.sliceDoc(r.from,Math.min(r.from+100,r.to)))[0].length;s&&e.head!=r.from+s&&(i=R.cursor(r.from+s))}return i}const aI=t=>tn(t,e=>Sr(t,e,!0)),cI=t=>tn(t,e=>Sr(t,e,!1)),uI=t=>tn(t,e=>Sr(t,e,!tt(t))),hI=t=>tn(t,e=>Sr(t,e,tt(t))),fI=t=>tn(t,e=>t.moveToLineBoundary(e,!1,!1)),dI=t=>tn(t,e=>t.moveToLineBoundary(e,!0,!1));function pI(t,e,n){let r=!1,i=ts(t.selection,s=>{let o=mn(t,s.head,-1)||mn(t,s.head,1)||s.head>0&&mn(t,s.head-1,1)||s.head<t.doc.length&&mn(t,s.head+1,-1);if(!o||!o.end)return s;r=!0;let l=o.start.from==s.head?o.end.to:o.end.from;return R.cursor(l)});return r?(e(en(t,i)),!0):!1}const mI=({state:t,dispatch:e})=>pI(t,e);function zt(t,e,n){let r=ts(t.state.selection,i=>{i.undirectional&&i.head>=i.anchor!=e&&(i=R.range(i.head,i.anchor));let s=n(i);return R.range(i.anchor,s.head,s.goalColumn,s.bidiLevel||void 0,s.assoc)});return r.eq(t.state.selection)?!1:(t.dispatch(en(t.state,r)),!0)}function T1(t,e){return zt(t,e,n=>t.moveByChar(n,e))}const R1=t=>T1(t,!tt(t)),M1=t=>T1(t,tt(t));function I1(t,e){return zt(t,e,n=>t.moveByGroup(n,e))}const gI=t=>I1(t,!tt(t)),vI=t=>I1(t,tt(t)),yI=t=>{let e=!tt(t);return zt(t,e,n=>hc(t.state,n,e))},wI=t=>{let e=tt(t);return zt(t,e,n=>hc(t.state,n,e))};function A1(t,e){return zt(t,e,n=>t.moveVertically(n,e))}const P1=t=>A1(t,!1),E1=t=>A1(t,!0);function L1(t,e){return zt(t,e,n=>t.moveVertically(n,e,C1(t).height))}const Cg=t=>L1(t,!1),bg=t=>L1(t,!0),SI=t=>zt(t,!0,e=>Sr(t,e,!0)),kI=t=>zt(t,!1,e=>Sr(t,e,!1)),xI=t=>{let e=!tt(t);return zt(t,e,n=>Sr(t,n,e))},CI=t=>{let e=tt(t);return zt(t,e,n=>Sr(t,n,e))},bI=t=>zt(t,!1,e=>R.cursor(t.lineBlockAt(e.head).from)),TI=t=>zt(t,!0,e=>R.cursor(t.lineBlockAt(e.head).to)),Tg=({state:t,dispatch:e})=>(e(en(t,{anchor:0})),!0),Rg=({state:t,dispatch:e})=>(e(en(t,{anchor:t.doc.length})),!0),Mg=({state:t,dispatch:e})=>(e(en(t,{anchor:t.selection.main.anchor,head:0})),!0),Ig=({state:t,dispatch:e})=>(e(en(t,{anchor:t.selection.main.anchor,head:t.doc.length})),!0),RI=({state:t,dispatch:e})=>(e(t.update({selection:{anchor:0,head:t.doc.length},userEvent:"select"})),!0),MI=({state:t,dispatch:e})=>{let n=fc(t).map(({from:r,to:i})=>R.undirectionalRange(r,Math.min(i+1,t.doc.length)));return e(t.update({selection:R.create(n),userEvent:"select"})),!0},II=({state:t,dispatch:e})=>{let n=ts(t.selection,r=>{let i=et(t),s=i.resolveStack(r.from,1);if(r.empty){let o=i.resolveStack(r.from,-1);o.node.from>=s.node.from&&o.node.to<=s.node.to&&(s=o)}for(let o=s;o;o=o.next){let{node:l}=o;if((l.from<r.from&&l.to>=r.to||l.to>r.to&&l.from<=r.from)&&o.next)return R.undirectionalRange(l.from,l.to)}return r});return n.eq(t.selection)?!1:(e(en(t,n)),!0)};function D1(t,e){let{state:n}=t,r=n.selection,i=n.selection.ranges.slice();for(let s of n.selection.ranges){let o=n.doc.lineAt(s.head);if(e?o.to<t.state.doc.length:o.from>0)for(let l=s;;){let a=t.moveVertically(l,e);if(a.head<o.from||a.head>o.to){i.some(c=>c.head==a.head)||i.push(a);break}else{if(a.head==l.head)break;l=a}}}return i.length==r.ranges.length?!1:(t.dispatch(en(n,R.create(i,i.length-1))),!0)}const AI=t=>D1(t,!1),PI=t=>D1(t,!0),EI=({state:t,dispatch:e})=>{let n=t.selection,r=null;return n.ranges.length>1?r=R.create([n.main]):n.main.empty||(r=R.create([R.cursor(n.main.head)])),r?(e(en(t,r)),!0):!1};function Lo(t,e){if(t.state.readOnly)return!1;let n="delete.selection",{state:r}=t,i=r.changeByRange(s=>{let{from:o,to:l}=s;if(o==l){let a=e(s);a<o?(n="delete.backward",a=gl(t,a,!1)):a>o&&(n="delete.forward",a=gl(t,a,!0)),o=Math.min(o,a),l=Math.max(l,a)}else o=gl(t,o,!1),l=gl(t,l,!0);return o==l?{range:s}:{changes:{from:o,to:l},range:R.cursor(o,o<s.head?-1:1)}});return i.changes.empty?!1:(t.dispatch(r.update(i,{scrollIntoView:!0,userEvent:n,effects:n=="delete.selection"?F.announce.of(r.phrase("Selection deleted")):void 0})),!0)}function gl(t,e,n){if(t instanceof F)for(let r of t.state.facet(F.atomicRanges).map(i=>i(t)))r.between(e,e,(i,s)=>{i<e&&s>e&&(e=n?s:i)});return e}const O1=(t,e,n)=>Lo(t,r=>{let i=r.from,{state:s}=t,o=s.doc.lineAt(i),l,a;if(n&&!e&&i>o.from&&i<o.from+200&&!/[^ \t]/.test(l=o.text.slice(0,i-o.from))){if(l[l.length-1]=="	")return i-1;let c=es(l,s.tabSize),u=c%Kr(s)||Kr(s);for(let h=0;h<u&&l[l.length-1-h]==" ";h++)i--;a=i}else a=qe(o.text,i-o.from,e,e)+o.from,a==i&&o.number!=(e?s.doc.lines:1)?a+=e?1:-1:!e&&/[\ufe00-\ufe0f]/.test(o.text.slice(a-o.from,i-o.from))&&(a=qe(o.text,a-o.from,!1,!1)+o.from);return a}),lf=t=>O1(t,!1,!0),N1=t=>O1(t,!0,!1),B1=(t,e)=>Lo(t,n=>{let r=n.head,{state:i}=t,s=i.doc.lineAt(r),o=i.charCategorizer(r);for(let l=null;;){if(r==(e?s.to:s.from)){r==n.head&&s.number!=(e?i.doc.lines:1)&&(r+=e?1:-1);break}let a=qe(s.text,r-s.from,e)+s.from,c=s.text.slice(Math.min(r,a)-s.from,Math.max(r,a)-s.from),u=o(c);if(l!=null&&u!=l)break;(c!=" "||r!=n.head)&&(l=u),r=a}return r}),_1=t=>B1(t,!1),LI=t=>B1(t,!0),DI=t=>Lo(t,e=>{let n=t.lineBlockAt(e.head).to;return e.head<n?n:Math.min(t.state.doc.length,e.head+1)}),OI=t=>Lo(t,e=>{let n=t.moveToLineBoundary(e,!1).head;return e.head>n?n:Math.max(0,e.head-1)}),NI=t=>Lo(t,e=>{let n=t.moveToLineBoundary(e,!0).head;return e.head<n?n:Math.min(t.state.doc.length,e.head+1)}),BI=({state:t,dispatch:e})=>{if(t.readOnly)return!1;let n=t.changeByRange(r=>({changes:{from:r.from,to:r.to,insert:te.of(["",""])},range:R.cursor(r.from)}));return e(t.update(n,{scrollIntoView:!0,userEvent:"input"})),!0},_I=({state:t,dispatch:e})=>{if(t.readOnly)return!1;let n=t.changeByRange(r=>{if(!r.empty||r.from==0||r.from==t.doc.length)return{range:r};let i=r.from,s=t.doc.lineAt(i),o=i==s.from?i-1:qe(s.text,i-s.from,!1)+s.from,l=i==s.to?i+1:qe(s.text,i-s.from,!0)+s.from;return{changes:{from:o,to:l,insert:t.doc.slice(i,l).append(t.doc.slice(o,i))},range:R.cursor(l)}});return n.changes.empty?!1:(e(t.update(n,{scrollIntoView:!0,userEvent:"move.character"})),!0)};function fc(t){let e=[],n=-1;for(let r of t.selection.ranges){let i=t.doc.lineAt(r.from),s=t.doc.lineAt(r.to);if(!r.empty&&r.to==s.from&&(s=t.doc.lineAt(r.to-1)),n>=i.number){let o=e[e.length-1];o.to=s.to,o.ranges.push(r)}else e.push({from:i.from,to:s.to,ranges:[r]});n=s.number+1}return e}function q1(t,e,n){if(t.readOnly)return!1;let r=[],i=[];for(let s of fc(t)){if(n?s.to==t.doc.length:s.from==0)continue;let o=t.doc.lineAt(n?s.to+1:s.from-1),l=o.length+1;if(n){r.push({from:s.to,to:o.to},{from:s.from,insert:o.text+t.lineBreak});for(let a of s.ranges)i.push(R.range(Math.min(t.doc.length,a.anchor+l),Math.min(t.doc.length,a.head+l)))}else{r.push({from:o.from,to:s.from},{from:s.to,insert:t.lineBreak+o.text});for(let a of s.ranges)i.push(R.range(a.anchor-l,a.head-l))}}return r.length?(e(t.update({changes:r,scrollIntoView:!0,selection:R.create(i,t.selection.mainIndex),userEvent:"move.line"})),!0):!1}const qI=({state:t,dispatch:e})=>q1(t,e,!1),WI=({state:t,dispatch:e})=>q1(t,e,!0);function W1(t,e,n){if(t.readOnly)return!1;let r=[];for(let s of fc(t))n?r.push({from:s.from,insert:t.doc.slice(s.from,s.to)+t.lineBreak}):r.push({from:s.to,insert:t.lineBreak+t.doc.slice(s.from,s.to)});let i=t.changes(r);return e(t.update({changes:i,selection:t.selection.map(i,n?1:-1),scrollIntoView:!0,userEvent:"input.copyline"})),!0}const FI=({state:t,dispatch:e})=>W1(t,e,!1),zI=({state:t,dispatch:e})=>W1(t,e,!0),jI=t=>{if(t.state.readOnly)return!1;let{state:e}=t,n=e.changes(fc(e).map(({from:i,to:s})=>(i>0?i--:s<e.doc.length&&s++,{from:i,to:s}))),r=ts(e.selection,i=>{let s;if(t.lineWrapping){let o=t.lineBlockAt(i.head),l=t.coordsAtPos(i.head,i.assoc||1);l&&(s=o.bottom+t.documentTop-l.bottom+t.defaultLineHeight/2)}return t.moveVertically(i,!0,s)}).map(n);return t.dispatch({changes:n,selection:r,scrollIntoView:!0,userEvent:"delete.line"}),!0};function VI(t,e){if(/\(\)|\[\]|\{\}/.test(t.sliceDoc(e-1,e+1)))return{from:e,to:e};let n=et(t).resolveInner(e),r=n.childBefore(e),i=n.childAfter(e),s;return r&&i&&r.to<=e&&i.from>=e&&(s=r.type.prop(Z.closedBy))&&s.indexOf(i.name)>-1&&t.doc.lineAt(r.to).from==t.doc.lineAt(i.from).from&&!/\S/.test(t.sliceDoc(r.to,i.from))?{from:r.to,to:i.from}:null}const Ag=F1(!1),HI=F1(!0);function F1(t){return({state:e,dispatch:n})=>{if(e.readOnly)return!1;let r=e.changeByRange(i=>{let{from:s,to:o}=i,l=e.doc.lineAt(s),a=!t&&s==o&&VI(e,s);t&&(s=o=(o<=l.to?l:e.doc.lineAt(o)).to);let c=new lc(e,{simulateBreak:s,simulateDoubleBreak:!!a}),u=Md(c,s);for(u==null&&(u=es(/^\s*/.exec(e.doc.lineAt(s).text)[0],e.tabSize));o<l.to&&/\s/.test(l.text[o-l.from]);)o++;a?{from:s,to:o}=a:s>l.from&&s<l.from+100&&!/\S/.test(l.text.slice(0,s))&&(s=l.from);let h=["",go(e,u)];return a&&h.push(go(e,c.lineIndent(l.from,-1))),{changes:{from:s,to:o,insert:te.of(h)},range:R.cursor(s+1+h[1].length)}});return n(e.update(r,{scrollIntoView:!0,userEvent:"input"})),!0}}function Od(t,e){let n=-1;return t.changeByRange(r=>{let i=[];for(let o=r.from;o<=r.to;){let l=t.doc.lineAt(o);l.number>n&&(r.empty||r.to>l.from)&&(e(l,i,r),n=l.number),o=l.to+1}let s=t.changes(i);return{changes:i,range:R.range(s.mapPos(r.anchor,1),s.mapPos(r.head,1))}})}const UI=({state:t,dispatch:e})=>{if(t.readOnly)return!1;let n=Object.create(null),r=new lc(t,{overrideIndentation:s=>{let o=n[s];return o??-1}}),i=Od(t,(s,o,l)=>{let a=Md(r,s.from);if(a==null)return;/\S/.test(s.text)||(a=0);let c=/^\s*/.exec(s.text)[0],u=go(t,a);(c!=u||l.from<s.from+c.length)&&(n[s.from]=a,o.push({from:s.from,to:s.from+c.length,insert:u}))});return i.changes.empty||e(t.update(i,{userEvent:"indent"})),!0},z1=({state:t,dispatch:e})=>t.readOnly?!1:(e(t.update(Od(t,(n,r)=>{r.push({from:n.from,insert:t.facet(Rd)})}),{userEvent:"input.indent"})),!0),j1=({state:t,dispatch:e})=>t.readOnly?!1:(e(t.update(Od(t,(n,r)=>{let i=/^\s*/.exec(n.text)[0];if(!i)return;let s=es(i,t.tabSize),o=0,l=go(t,Math.max(0,s-Kr(t)));for(;o<i.length&&o<l.length&&i.charCodeAt(o)==l.charCodeAt(o);)o++;r.push({from:n.from+o,to:n.from+i.length,insert:l.slice(o)})}),{userEvent:"delete.dedent"})),!0),$I=t=>(t.setTabFocusMode(),!0),GI=[{key:"Ctrl-b",run:v1,shift:R1,preventDefault:!0},{key:"Ctrl-f",run:y1,shift:M1},{key:"Ctrl-p",run:k1,shift:P1},{key:"Ctrl-n",run:x1,shift:E1},{key:"Ctrl-a",run:fI,shift:bI},{key:"Ctrl-e",run:dI,shift:TI},{key:"Ctrl-d",run:N1},{key:"Ctrl-h",run:lf},{key:"Ctrl-k",run:DI},{key:"Ctrl-Alt-h",run:_1},{key:"Ctrl-o",run:BI},{key:"Ctrl-t",run:_I},{key:"Ctrl-v",run:of}],KI=[{key:"ArrowLeft",run:v1,shift:R1,preventDefault:!0},{key:"Mod-ArrowLeft",mac:"Alt-ArrowLeft",run:rI,shift:gI,preventDefault:!0},{mac:"Cmd-ArrowLeft",run:uI,shift:xI,preventDefault:!0},{key:"ArrowRight",run:y1,shift:M1,preventDefault:!0},{key:"Mod-ArrowRight",mac:"Alt-ArrowRight",run:iI,shift:vI,preventDefault:!0},{mac:"Cmd-ArrowRight",run:hI,shift:CI,preventDefault:!0},{key:"ArrowUp",run:k1,shift:P1,preventDefault:!0},{mac:"Cmd-ArrowUp",run:Tg,shift:Mg},{mac:"Ctrl-ArrowUp",run:xg,shift:Cg},{key:"ArrowDown",run:x1,shift:E1,preventDefault:!0},{mac:"Cmd-ArrowDown",run:Rg,shift:Ig},{mac:"Ctrl-ArrowDown",run:of,shift:bg},{key:"PageUp",run:xg,shift:Cg},{key:"PageDown",run:of,shift:bg},{key:"Home",run:cI,shift:kI,preventDefault:!0},{key:"Mod-Home",run:Tg,shift:Mg},{key:"End",run:aI,shift:SI,preventDefault:!0},{key:"Mod-End",run:Rg,shift:Ig},{key:"Enter",run:Ag,shift:Ag},{key:"Mod-a",run:RI},{key:"Backspace",run:lf,shift:lf,preventDefault:!0},{key:"Delete",run:N1,preventDefault:!0},{key:"Mod-Backspace",mac:"Alt-Backspace",run:_1,preventDefault:!0},{key:"Mod-Delete",mac:"Alt-Delete",run:LI,preventDefault:!0},{mac:"Mod-Backspace",run:OI,preventDefault:!0},{mac:"Mod-Delete",run:NI,preventDefault:!0}].concat(GI.map(t=>({mac:t.key,run:t.run,shift:t.shift}))),QI=[{key:"Alt-ArrowLeft",mac:"Ctrl-ArrowLeft",run:oI,shift:yI},{key:"Alt-ArrowRight",mac:"Ctrl-ArrowRight",run:lI,shift:wI},{key:"Alt-ArrowUp",run:qI},{key:"Shift-Alt-ArrowUp",run:FI},{key:"Alt-ArrowDown",run:WI},{key:"Shift-Alt-ArrowDown",run:zI},{key:"Mod-Alt-ArrowUp",run:AI},{key:"Mod-Alt-ArrowDown",run:PI},{key:"Escape",run:EI},{key:"Mod-Enter",run:HI},{key:"Alt-l",mac:"Ctrl-l",run:MI},{key:"Mod-i",run:II,preventDefault:!0},{key:"Mod-[",run:j1},{key:"Mod-]",run:z1},{key:"Mod-Alt-\\",run:UI},{key:"Shift-Mod-k",run:jI},{key:"Shift-Mod-\\",run:mI},{key:"Mod-/",run:q2},{key:"Alt-A",mac:"Ctrl-A",run:F2},{key:"Ctrl-m",mac:"Shift-Alt-m",run:$I}].concat(KI),YI={key:"Tab",run:z1,shift:j1},Pg=typeof String.prototype.normalize=="function"?t=>t.normalize("NFKD"):t=>t;class Yi{constructor(e,n,r=0,i=e.length,s,o){this.test=o,this.value={from:0,to:0,precise:!1},this.done=!1,this.matches=[],this.buffer="",this.bufferPos=0,this.iter=e.iterRange(r,i),this.bufferStart=r,this.normalize=s?l=>s(Pg(l)):Pg,this.query=this.normalize(n)}peek(){if(this.bufferPos==this.buffer.length){if(this.bufferStart+=this.buffer.length,this.iter.next(),this.iter.done)return-1;this.bufferPos=0,this.buffer=this.iter.value}return ft(this.buffer,this.bufferPos)}next(){for(;this.matches.length;)this.matches.pop();return this.nextOverlapping()}nextOverlapping(){for(;;){let e=this.peek();if(e<0)return this.done=!0,this;let n=sd(e),r=this.bufferStart+this.bufferPos;this.bufferPos+=fn(e);let i=this.normalize(n);if(i.length)for(let s=0,o=r,l=!0;;s++){let a=i.charCodeAt(s),c=this.match(a,o,l,this.bufferPos+this.bufferStart,s==i.length-1);if(c)return this.value=c,this;if(s==i.length-1)break;l&&s<n.length&&n.charCodeAt(s)==a?o++:l=!1}}}match(e,n,r,i,s){let o=null;for(let l=0;l<this.matches.length;){let a=this.matches[l],c=!1;this.query.charCodeAt(a.index)==e&&(a.index==this.query.length-1?o={from:a.from,to:i,precise:s&&a.precise}:(a.index++,c=!0)),c?l++:this.matches.splice(l,1)}return this.query.charCodeAt(0)==e&&(this.query.length==1?o={from:n,to:i,precise:r&&s}:this.matches.push({from:n,index:1,precise:r})),o&&this.test&&!this.test(o.from,o.to,this.buffer,this.bufferStart)&&(o=null),o}}typeof Symbol<"u"&&(Yi.prototype[Symbol.iterator]=function(){return this});const V1={from:-1,to:-1,match:/.*/.exec(""),precise:!0},Nd="gm"+(/x/.unicode==null?"":"u");class H1{constructor(e,n,r,i=0,s=e.length){if(this.text=e,this.to=s,this.curLine="",this.done=!1,this.value=V1,/\\[sWDnr]|\n|\r|\[\^/.test(n))return new U1(e,n,r,i,s);this.re=new RegExp(n,Nd+(r!=null&&r.ignoreCase?"i":"")),this.test=r==null?void 0:r.test,this.iter=e.iter();let o=e.lineAt(i);this.curLineStart=o.from,this.matchPos=Ia(e,i),this.getLine(this.curLineStart)}getLine(e){this.iter.next(e),this.iter.lineBreak?this.curLine="":(this.curLine=this.iter.value,this.curLineStart+this.curLine.length>this.to&&(this.curLine=this.curLine.slice(0,this.to-this.curLineStart)),this.iter.next())}nextLine(){this.curLineStart=this.curLineStart+this.curLine.length+1,this.curLineStart>this.to?this.curLine="":this.getLine(0)}next(){for(let e=this.matchPos-this.curLineStart;;){this.re.lastIndex=e;let n=this.matchPos<=this.to&&this.re.exec(this.curLine);if(n){let r=this.curLineStart+n.index,i=r+n[0].length;if(this.matchPos=Ia(this.text,i+(r==i?1:0)),r==this.curLineStart+this.curLine.length&&this.nextLine(),(r<i||r>this.value.to)&&(!this.test||this.test(r,i,n)))return this.value={from:r,to:i,precise:!0,match:n},this;e=this.matchPos-this.curLineStart}else if(this.curLineStart+this.curLine.length<this.to)this.nextLine(),e=0;else return this.done=!0,this}}}const du=new WeakMap;class Li{constructor(e,n){this.from=e,this.text=n}get to(){return this.from+this.text.length}static get(e,n,r){let i=du.get(e);if(!i||i.from>=r||i.to<=n){let l=new Li(n,e.sliceString(n,r));return du.set(e,l),l}if(i.from==n&&i.to==r)return i;let{text:s,from:o}=i;return o>n&&(s=e.sliceString(n,o)+s,o=n),i.to<r&&(s+=e.sliceString(i.to,r)),du.set(e,new Li(o,s)),new Li(n,s.slice(n-o,r-o))}}class U1{constructor(e,n,r,i,s){this.text=e,this.to=s,this.done=!1,this.value=V1,this.matchPos=Ia(e,i),this.re=new RegExp(n,Nd+(r!=null&&r.ignoreCase?"i":"")),this.test=r==null?void 0:r.test,this.flat=Li.get(e,i,this.chunkEnd(i+5e3))}chunkEnd(e){return e>=this.to?this.to:this.text.lineAt(e).to}next(){for(;;){let e=this.re.lastIndex=this.matchPos-this.flat.from,n=this.re.exec(this.flat.text);if(n&&!n[0]&&n.index==e&&(this.re.lastIndex=e+1,n=this.re.exec(this.flat.text)),n){let r=this.flat.from+n.index,i=r+n[0].length;if((this.flat.to>=this.to||n.index+n[0].length<=this.flat.text.length-10)&&(!this.test||this.test(r,i,n)))return this.value={from:r,to:i,precise:!0,match:n},this.matchPos=Ia(this.text,i+(r==i?1:0)),this}if(this.flat.to==this.to)return this.done=!0,this;this.flat=Li.get(this.text,this.flat.from,this.chunkEnd(this.flat.from+this.flat.text.length*2))}}}typeof Symbol<"u"&&(H1.prototype[Symbol.iterator]=U1.prototype[Symbol.iterator]=function(){return this});function XI(t){try{return new RegExp(t,Nd),!0}catch{return!1}}function Ia(t,e){if(e>=t.length)return e;let n=t.lineAt(e),r;for(;e<n.to&&(r=n.text.charCodeAt(e-n.from))>=56320&&r<57344;)e++;return e}const JI=t=>{let e=fM(t,"cm-goto-line");if(e){let o=e.dom.querySelector("input[type=text]");return o&&o.select(),!0}let{state:n}=t,r=String(n.doc.lineAt(t.state.selection.main.head).number),{close:i,result:s}=hM(t,{class:"cm-goto-line",label:n.phrase("Go to line"),input:{type:"text",name:"line",value:r},focus:!0,submitLabel:n.phrase("go")});return s.then(o=>{let l=o&&/^([+-])?(\d+)?(:\d+)?(%)?$/.exec(o.elements.line.value);if(!l){t.dispatch({effects:i});return}let a=n.doc.lineAt(n.selection.main.head),[,c,u,h,f]=l,d=h?+h.slice(1):0,p=u?+u:a.number;if(u&&f){let g=p/100;c&&(g=g*(c=="-"?-1:1)+a.number/n.doc.lines),p=Math.round(n.doc.lines*g)}else u&&c&&(p=p*(c=="-"?-1:1)+a.number);let m=n.doc.line(Math.max(1,Math.min(n.doc.lines,p))),w=R.cursor(m.from+Math.max(0,Math.min(d,m.length)));t.dispatch({effects:[i,F.scrollIntoView(w.from,{y:"center"})],selection:w})}),!0},ZI={highlightWordAroundCursor:!1,minSelectionLength:1,maxMatches:100,wholeWords:!1},eA=z.define({combine(t){return Zt(t,ZI,{highlightWordAroundCursor:(e,n)=>e||n,minSelectionLength:Math.min,maxMatches:Math.min})}});function tA(t){return[oA,sA]}const nA=U.mark({class:"cm-selectionMatch"}),rA=U.mark({class:"cm-selectionMatch cm-selectionMatch-main"});function Eg(t,e,n,r){return(n==0||t(e.sliceDoc(n-1,n))!=we.Word)&&(r==e.doc.length||t(e.sliceDoc(r,r+1))!=we.Word)}function iA(t,e,n,r){return t(e.sliceDoc(n,n+1))==we.Word&&t(e.sliceDoc(r-1,r))==we.Word}const sA=Me.fromClass(class{constructor(t){this.decorations=this.getDeco(t)}update(t){(t.selectionSet||t.docChanged||t.viewportChanged)&&(this.decorations=this.getDeco(t.view))}getDeco(t){let e=t.state.facet(eA),{state:n}=t,r=n.selection;if(r.ranges.length>1)return U.none;let i=r.main,s,o=null;if(i.empty){if(!e.highlightWordAroundCursor)return U.none;let a=n.wordAt(i.head);if(!a)return U.none;o=n.charCategorizer(i.head),s=n.sliceDoc(a.from,a.to)}else{let a=i.to-i.from;if(a<e.minSelectionLength||a>200)return U.none;if(e.wholeWords){if(s=n.sliceDoc(i.from,i.to),o=n.charCategorizer(i.head),!(Eg(o,n,i.from,i.to)&&iA(o,n,i.from,i.to)))return U.none}else if(s=n.sliceDoc(i.from,i.to),!s)return U.none}let l=[];for(let a of t.visibleRanges){let c=new Yi(n.doc,s,a.from,a.to);for(;!c.next().done;){let{from:u,to:h}=c.value;if((!o||Eg(o,n,u,h))&&(i.empty&&u<=i.from&&h>=i.to?l.push(rA.range(u,h)):(u>=i.to||h<=i.from)&&l.push(nA.range(u,h)),l.length>e.maxMatches))return U.none}}return U.set(l)}},{decorations:t=>t.decorations}),oA=F.baseTheme({".cm-selectionMatch":{backgroundColor:"#99ff7780"},".cm-searchMatch .cm-selectionMatch":{backgroundColor:"transparent"}}),lA=({state:t,dispatch:e})=>{let{selection:n}=t,r=R.create(n.ranges.map(i=>t.wordAt(i.head)||R.cursor(i.head)),n.mainIndex);return r.eq(n)?!1:(e(t.update({selection:r})),!0)};function aA(t,e){let{main:n,ranges:r}=t.selection,i=t.wordAt(n.head),s=i&&i.from==n.from&&i.to==n.to;for(let o=!1,l=new Yi(t.doc,e,r[r.length-1].to);;)if(l.next(),l.done){if(o)return null;l=new Yi(t.doc,e,0,Math.max(0,r[r.length-1].from-1)),o=!0}else{if(o&&r.some(a=>a.from==l.value.from))continue;if(s){let a=t.wordAt(l.value.from);if(!a||a.from!=l.value.from||a.to!=l.value.to)continue}return l.value}}const cA=({state:t,dispatch:e})=>{let{ranges:n}=t.selection;if(n.some(s=>s.from===s.to))return lA({state:t,dispatch:e});let r=t.sliceDoc(n[0].from,n[0].to);if(t.selection.ranges.some(s=>t.sliceDoc(s.from,s.to)!=r))return!1;let i=aA(t,r);return i?(e(t.update({selection:t.selection.addRange(R.range(i.from,i.to),!1),effects:F.scrollIntoView(i.to)})),!0):!1},ns=z.define({combine(t){return Zt(t,{top:!1,caseSensitive:!1,literal:!1,regexp:!1,wholeWord:!1,createPanel:e=>new CA(e),scrollToMatch:e=>F.scrollIntoView(e)})}});class $1{constructor(e){this.search=e.search,this.caseSensitive=!!e.caseSensitive,this.literal=!!e.literal,this.regexp=!!e.regexp,this.replace=e.replace||"",this.valid=!!this.search&&(!this.regexp||XI(this.search)),this.unquoted=this.unquote(this.search),this.wholeWord=!!e.wholeWord,this.test=e.test}unquote(e){return this.literal?e:e.replace(/\\([nrt\\])/g,(n,r)=>r=="n"?`
`:r=="r"?"\r":r=="t"?"	":"\\")}eq(e){return this.search==e.search&&this.replace==e.replace&&this.caseSensitive==e.caseSensitive&&this.regexp==e.regexp&&this.wholeWord==e.wholeWord&&this.test==e.test}create(){return this.regexp?new mA(this):new fA(this)}getCursor(e,n=0,r){let i=e.doc?e:ee.create({doc:e});return r==null&&(r=i.doc.length),this.regexp?si(this,i,n,r):ii(this,i,n,r)}}class G1{constructor(e){this.spec=e}}function uA(t,e,n){return(r,i,s,o)=>{if(n&&!n(r,i,s,o))return!1;let l=r>=o&&i<=o+s.length?s.slice(r-o,i-o):e.doc.sliceString(r,i);return t(l,e,r,i)}}function ii(t,e,n,r){let i;return t.wholeWord&&(i=hA(e.doc,e.charCategorizer(e.selection.main.head))),t.test&&(i=uA(t.test,e,i)),new Yi(e.doc,t.unquoted,n,r,t.caseSensitive?void 0:s=>s.toLowerCase(),i)}function hA(t,e){return(n,r,i,s)=>((s>n||s+i.length<r)&&(s=Math.max(0,n-2),i=t.sliceString(s,Math.min(t.length,r+2))),(e(Aa(i,n-s))!=we.Word||e(Pa(i,n-s))!=we.Word)&&(e(Pa(i,r-s))!=we.Word||e(Aa(i,r-s))!=we.Word))}class fA extends G1{constructor(e){super(e)}nextMatch(e,n,r){let i=ii(this.spec,e,r,e.doc.length).nextOverlapping();if(i.done){let s=Math.min(e.doc.length,n+this.spec.unquoted.length);i=ii(this.spec,e,0,s).nextOverlapping()}return i.done||i.value.from==n&&i.value.to==r?null:i.value}prevMatchInRange(e,n,r){for(let i=r;;){let s=Math.max(n,i-1e4-this.spec.unquoted.length),o=ii(this.spec,e,s,i),l=null;for(;!o.nextOverlapping().done;)l=o.value;if(l)return l;if(s==n)return null;i-=1e4}}prevMatch(e,n,r){let i=this.prevMatchInRange(e,0,n);return i||(i=this.prevMatchInRange(e,Math.max(0,r-this.spec.unquoted.length),e.doc.length)),i&&(i.from!=n||i.to!=r)?i:null}getReplacement(e){return this.spec.unquote(this.spec.replace)}matchAll(e,n){let r=ii(this.spec,e,0,e.doc.length),i=[];for(;!r.next().done;){if(i.length>=n)return null;i.push(r.value)}return i}highlight(e,n,r,i){let s=ii(this.spec,e,Math.max(0,n-this.spec.unquoted.length),Math.min(r+this.spec.unquoted.length,e.doc.length));for(;!s.next().done;)i(s.value.from,s.value.to)}}function dA(t,e,n){return(r,i,s)=>(!n||n(r,i,s))&&t(s[0],e,r,i)}function si(t,e,n,r){let i;return t.wholeWord&&(i=pA(e.charCategorizer(e.selection.main.head))),t.test&&(i=dA(t.test,e,i)),new H1(e.doc,t.search,{ignoreCase:!t.caseSensitive,test:i},n,r)}function Aa(t,e){return t.slice(qe(t,e,!1),e)}function Pa(t,e){return t.slice(e,qe(t,e))}function pA(t){return(e,n,r)=>!r[0].length||(t(Aa(r.input,r.index))!=we.Word||t(Pa(r.input,r.index))!=we.Word)&&(t(Pa(r.input,r.index+r[0].length))!=we.Word||t(Aa(r.input,r.index+r[0].length))!=we.Word)}class mA extends G1{nextMatch(e,n,r){let i=si(this.spec,e,r,e.doc.length).next();return i.done&&(i=si(this.spec,e,0,n).next()),i.done?null:i.value}prevMatchInRange(e,n,r){for(let i=1;;i++){let s=Math.max(n,r-i*1e4),o=si(this.spec,e,s,r),l=null;for(;!o.next().done;)l=o.value;if(l&&(s==n||l.from>s+10))return l;if(s==n)return null}}prevMatch(e,n,r){return this.prevMatchInRange(e,0,n)||this.prevMatchInRange(e,r,e.doc.length)}getReplacement(e){return this.spec.unquote(this.spec.replace).replace(/\$([$&]|\d+)/g,(n,r)=>{if(r=="&")return e.match[0];if(r=="$")return"$";for(let i=r.length;i>0;i--){let s=+r.slice(0,i);if(s>0&&s<e.match.length)return e.match[s]+r.slice(i)}return n})}matchAll(e,n){let r=si(this.spec,e,0,e.doc.length),i=[];for(;!r.next().done;){if(i.length>=n)return null;i.push(r.value)}return i}highlight(e,n,r,i){let s=si(this.spec,e,Math.max(0,n-250),Math.min(r+250,e.doc.length));for(;!s.next().done;)i(s.value.from,s.value.to)}}const yo=K.define(),Bd=K.define(),lr=ze.define({create(t){return new pu(af(t).create(),null)},update(t,e){for(let n of e.effects)n.is(yo)?t=new pu(n.value.create(),t.panel):n.is(Bd)&&(t=new pu(t.query,n.value?_d:null));return t},provide:t=>fo.from(t,e=>e.panel)});class pu{constructor(e,n){this.query=e,this.panel=n}}const gA=U.mark({class:"cm-searchMatch"}),vA=U.mark({class:"cm-searchMatch cm-searchMatch-selected"}),yA=Me.fromClass(class{constructor(t){this.view=t,this.decorations=this.highlight(t.state.field(lr))}update(t){let e=t.state.field(lr);(e!=t.startState.field(lr)||t.docChanged||t.selectionSet||t.viewportChanged)&&(this.decorations=this.highlight(e))}highlight({query:t,panel:e}){if(!e||!t.spec.valid)return U.none;let{view:n}=this,r=new xn;for(let i=0,s=n.visibleRanges,o=s.length;i<o;i++){let{from:l,to:a}=s[i];for(;i<o-1&&a>s[i+1].from-2*250;)a=s[++i].to;t.highlight(n.state,l,a,(c,u)=>{let h=n.state.selection.ranges.some(f=>f.from==c&&f.to==u);r.add(c,u,h?vA:gA)})}return r.finish()}},{decorations:t=>t.decorations});function Do(t){return e=>{let n=e.state.field(lr,!1);return n&&n.query.spec.valid?t(e,n):Y1(e)}}const Ea=Do((t,{query:e})=>{let{to:n}=t.state.selection.main,r=e.nextMatch(t.state,n,n);if(!r)return!1;let i=R.single(r.from,r.to),s=t.state.facet(ns);return t.dispatch({selection:i,effects:[qd(t,r),s.scrollToMatch(i.main,t)],userEvent:"select.search"}),Q1(t),!0}),La=Do((t,{query:e})=>{let{state:n}=t,{from:r}=n.selection.main,i=e.prevMatch(n,r,r);if(!i)return!1;let s=R.single(i.from,i.to),o=t.state.facet(ns);return t.dispatch({selection:s,effects:[qd(t,i),o.scrollToMatch(s.main,t)],userEvent:"select.search"}),Q1(t),!0}),wA=Do((t,{query:e})=>{let n=e.matchAll(t.state,1e3);return!n||!n.length?!1:(t.dispatch({selection:R.create(n.map(r=>R.range(r.from,r.to))),userEvent:"select.search.matches"}),!0)}),SA=({state:t,dispatch:e})=>{let n=t.selection;if(n.ranges.length>1||n.main.empty)return!1;let{from:r,to:i}=n.main,s=[],o=0;for(let l=new Yi(t.doc,t.sliceDoc(r,i));!l.next().done;){if(s.length>1e3)return!1;l.value.from==r&&(o=s.length),s.push(R.range(l.value.from,l.value.to))}return e(t.update({selection:R.create(s,o),userEvent:"select.search.matches"})),!0},Lg=Do((t,{query:e})=>{let{state:n}=t,{from:r,to:i}=n.selection.main;if(n.readOnly)return!1;let s=e.nextMatch(n,r,r);if(!s)return!1;let o=s,l=[],a,c,u=[];o.precise?o.from==r&&o.to==i&&(c=n.toText(e.getReplacement(o)),l.push({from:o.from,to:o.to,insert:c}),o=e.nextMatch(n,o.from,o.to),u.push(F.announce.of(n.phrase("replaced match on line $",n.doc.lineAt(r).number)+"."))):o=e.nextMatch(n,o.from,o.to);let h=t.state.changes(l);return o&&(a=R.single(o.from,o.to).map(h),u.push(qd(t,o)),u.push(n.facet(ns).scrollToMatch(a.main,t))),t.dispatch({changes:h,selection:a,effects:u,userEvent:"input.replace"}),!0}),kA=Do((t,{query:e})=>{if(t.state.readOnly)return!1;let n=[];for(let i of e.matchAll(t.state,1e9)){let{from:s,to:o,precise:l}=i;l&&n.push({from:s,to:o,insert:e.getReplacement(i)})}if(!n.length)return!1;let r=t.state.phrase("replaced $ matches",n.length)+".";return t.dispatch({changes:n,effects:F.announce.of(r),userEvent:"input.replace.all"}),!0});function _d(t){return t.state.facet(ns).createPanel(t)}function af(t,e){var n,r,i,s,o;let l=t.selection.main,a=l.empty||l.to>l.from+100?"":t.sliceDoc(l.from,l.to);if(e&&!a)return e;let c=t.facet(ns);return new $1({search:((n=e==null?void 0:e.literal)!==null&&n!==void 0?n:c.literal)?a:a.replace(/\n/g,"\\n"),caseSensitive:(r=e==null?void 0:e.caseSensitive)!==null&&r!==void 0?r:c.caseSensitive,literal:(i=e==null?void 0:e.literal)!==null&&i!==void 0?i:c.literal,regexp:(s=e==null?void 0:e.regexp)!==null&&s!==void 0?s:c.regexp,wholeWord:(o=e==null?void 0:e.wholeWord)!==null&&o!==void 0?o:c.wholeWord})}function K1(t){let e=oc(t,_d);return e&&e.dom.querySelector("[main-field]")}function Q1(t){let e=K1(t);e&&e==t.root.activeElement&&e.select()}const Y1=t=>{let e=t.state.field(lr,!1);if(e&&e.panel){let n=K1(t);if(n&&n!=t.root.activeElement){let r=af(t.state,e.query.spec);r.valid&&t.dispatch({effects:yo.of(r)}),n.focus(),n.select()}}else t.dispatch({effects:[Bd.of(!0),e?yo.of(af(t.state,e.query.spec)):K.appendConfig.of(TA)]});return!0},X1=t=>{let e=t.state.field(lr,!1);if(!e||!e.panel)return!1;let n=oc(t,_d);return n&&n.dom.contains(t.root.activeElement)&&t.focus(),t.dispatch({effects:Bd.of(!1)}),!0},xA=[{key:"Mod-f",run:Y1,scope:"editor search-panel"},{key:"F3",run:Ea,shift:La,scope:"editor search-panel",preventDefault:!0},{key:"Mod-g",run:Ea,shift:La,scope:"editor search-panel",preventDefault:!0},{key:"Escape",run:X1,scope:"editor search-panel"},{key:"Mod-Shift-l",run:SA},{key:"Mod-Alt-g",run:JI},{key:"Mod-d",run:cA,preventDefault:!0}];class CA{constructor(e){this.view=e;let n=this.query=e.state.field(lr).query.spec;this.commit=this.commit.bind(this),this.searchField=le("input",{value:n.search,placeholder:kt(e,"Find"),"aria-label":kt(e,"Find"),class:"cm-textfield",name:"search",form:"","main-field":"true",onchange:this.commit,onkeyup:this.commit}),this.replaceField=le("input",{value:n.replace,placeholder:kt(e,"Replace"),"aria-label":kt(e,"Replace"),class:"cm-textfield",name:"replace",form:"",onchange:this.commit,onkeyup:this.commit}),this.caseField=le("input",{type:"checkbox",name:"case",form:"",checked:n.caseSensitive,onchange:this.commit}),this.reField=le("input",{type:"checkbox",name:"re",form:"",checked:n.regexp,onchange:this.commit}),this.wordField=le("input",{type:"checkbox",name:"word",form:"",checked:n.wholeWord,onchange:this.commit});function r(i,s,o){return le("button",{class:"cm-button",name:i,onclick:s,type:"button"},o)}this.dom=le("div",{onkeydown:i=>this.keydown(i),class:"cm-search"},[this.searchField,r("next",()=>Ea(e),[kt(e,"next")]),r("prev",()=>La(e),[kt(e,"previous")]),r("select",()=>wA(e),[kt(e,"all")]),le("label",null,[this.caseField,kt(e,"match case")]),le("label",null,[this.reField,kt(e,"regexp")]),le("label",null,[this.wordField,kt(e,"by word")]),...e.state.readOnly?[]:[le("br"),this.replaceField,r("replace",()=>Lg(e),[kt(e,"replace")]),r("replaceAll",()=>kA(e),[kt(e,"replace all")])],le("button",{name:"close",onclick:()=>X1(e),"aria-label":kt(e,"close"),type:"button"},["×"])])}commit(){let e=new $1({search:this.searchField.value,caseSensitive:this.caseField.checked,regexp:this.reField.checked,wholeWord:this.wordField.checked,replace:this.replaceField.value});e.eq(this.query)||(this.query=e,this.view.dispatch({effects:yo.of(e)}))}keydown(e){kR(this.view,e,"search-panel")?e.preventDefault():e.keyCode==13&&e.target==this.searchField?(e.preventDefault(),(e.shiftKey?La:Ea)(this.view)):e.keyCode==13&&e.target==this.replaceField&&(e.preventDefault(),Lg(this.view))}update(e){for(let n of e.transactions)for(let r of n.effects)r.is(yo)&&!r.value.eq(this.query)&&this.setQuery(r.value)}setQuery(e){this.query=e,this.searchField.value=e.search,this.replaceField.value=e.replace,this.caseField.checked=e.caseSensitive,this.reField.checked=e.regexp,this.wordField.checked=e.wholeWord}mount(){this.searchField.select()}get pos(){return 80}get top(){return this.view.state.facet(ns).top}}function kt(t,e){return t.state.phrase(e)}const vl=30,yl=/[\s\.,:;?!]/;function qd(t,{from:e,to:n}){let r=t.state.doc.lineAt(e),i=t.state.doc.lineAt(n).to,s=Math.max(r.from,e-vl),o=Math.min(i,n+vl),l=t.state.sliceDoc(s,o);if(s!=r.from){for(let a=0;a<vl;a++)if(!yl.test(l[a+1])&&yl.test(l[a])){l=l.slice(a);break}}if(o!=i){for(let a=l.length-1;a>l.length-vl;a--)if(!yl.test(l[a-1])&&yl.test(l[a])){l=l.slice(0,a);break}}return F.announce.of(`${t.state.phrase("current match")}. ${l} ${t.state.phrase("on line")} ${r.number}.`)}const bA=F.baseTheme({".cm-panel.cm-search":{padding:"2px 6px 4px",position:"relative","& [name=close]":{position:"absolute",top:"0",right:"4px",backgroundColor:"inherit",border:"none",font:"inherit",padding:0,margin:0},"& input, & button, & label":{margin:".2em .6em .2em 0"},"& input[type=checkbox]":{marginRight:".2em"},"& label":{fontSize:"80%",whiteSpace:"pre"}},"&light .cm-searchMatch":{backgroundColor:"#ffff0054"},"&dark .cm-searchMatch":{backgroundColor:"#00ffff8a"},"&light .cm-searchMatch-selected":{backgroundColor:"#ff6a0054"},"&dark .cm-searchMatch-selected":{backgroundColor:"#ff00ff8a"}}),TA=[lr,Zr.low(yA),bA];class J1{constructor(e,n,r,i){this.state=e,this.pos=n,this.explicit=r,this.view=i,this.abortListeners=[],this.abortOnDocChange=!1}tokenBefore(e){let n=et(this.state).resolveInner(this.pos,-1);for(;n&&e.indexOf(n.name)<0;)n=n.parent;return n?{from:n.from,to:this.pos,text:this.state.sliceDoc(n.from,this.pos),type:n.type}:null}matchBefore(e){let n=this.state.doc.lineAt(this.pos),r=Math.max(n.from,this.pos-250),i=n.text.slice(r-n.from,this.pos-n.from),s=i.search(Z1(e,!1));return s<0?null:{from:r+s,to:this.pos,text:i.slice(s)}}get aborted(){return this.abortListeners==null}addEventListener(e,n,r){e=="abort"&&this.abortListeners&&(this.abortListeners.push(n),r&&r.onDocChange&&(this.abortOnDocChange=!0))}}function Dg(t){let e=Object.keys(t).join(""),n=/\w/.test(e);return n&&(e=e.replace(/\w/g,"")),`[${n?"\\w":""}${e.replace(/[^\w\s]/g,"\\$&")}]`}function RA(t){let e=Object.create(null),n=Object.create(null);for(let{label:i}of t){e[i[0]]=!0;for(let s=1;s<i.length;s++)n[i[s]]=!0}let r=Dg(e)+Dg(n)+"*$";return[new RegExp("^"+r),new RegExp(r)]}function MA(t){let e=t.map(i=>typeof i=="string"?{label:i}:i),[n,r]=e.every(i=>/^\w+$/.test(i.label))?[/\w*$/,/\w+$/]:RA(e);return i=>{let s=i.matchBefore(r);return s||i.explicit?{from:s?s.from:i.pos,options:e,validFor:n}:null}}class Og{constructor(e,n,r,i){this.completion=e,this.source=n,this.match=r,this.score=i}}function Wr(t){return t.selection.main.from}function Z1(t,e){var n;let{source:r}=t,i=e&&r[0]!="^",s=r[r.length-1]!="$";return!i&&!s?t:new RegExp(`${i?"^":""}(?:${r})${s?"$":""}`,(n=t.flags)!==null&&n!==void 0?n:t.ignoreCase?"i":"")}const eS=_n.define();function IA(t,e,n,r){let{main:i}=t.selection,s=n-i.from,o=r-i.from;return{...t.changeByRange(l=>{if(l!=i&&n!=r&&t.sliceDoc(l.from+s,l.from+o)!=t.sliceDoc(n,r))return{range:l};let a=t.toText(e);return{changes:{from:l.from+s,to:r==i.from?l.to:l.from+o,insert:a},range:R.cursor(l.from+s+a.length)}}),scrollIntoView:!0,userEvent:"input.complete"}}const Ng=new WeakMap;function AA(t){if(!Array.isArray(t))return t;let e=Ng.get(t);return e||Ng.set(t,e=MA(t)),e}const Da=K.define(),wo=K.define();class PA{constructor(e){this.pattern=e,this.chars=[],this.folded=[],this.any=[],this.precise=[],this.byWord=[],this.score=0,this.matched=[];for(let n=0;n<e.length;){let r=ft(e,n),i=fn(r);this.chars.push(r);let s=e.slice(n,n+i),o=s.toUpperCase();this.folded.push(ft(o==s?s.toLowerCase():o,0)),n+=i}this.astral=e.length!=this.chars.length}ret(e,n){return this.score=e,this.matched=n,this}match(e){if(this.pattern.length==0)return this.ret(-100,[]);if(e.length<this.pattern.length)return null;let{chars:n,folded:r,any:i,precise:s,byWord:o}=this;if(n.length==1){let v=ft(e,0),y=fn(v),x=y==e.length?0:-100;if(v!=n[0])if(v==r[0])x+=-200;else return null;return this.ret(x,[0,y])}let l=e.indexOf(this.pattern);if(l==0)return this.ret(e.length==this.pattern.length?0:-100,[0,this.pattern.length]);let a=n.length,c=0;if(l<0){for(let v=0,y=Math.min(e.length,200);v<y&&c<a;){let x=ft(e,v);(x==n[c]||x==r[c])&&(i[c++]=v),v+=fn(x)}if(c<a)return null}let u=0,h=0,f=!1,d=0,p=-1,m=-1,w=/[a-z]/.test(e),g=!0;for(let v=0,y=Math.min(e.length,200),x=0;v<y&&h<a;){let k=ft(e,v);l<0&&(u<a&&k==n[u]&&(s[u++]=v),d<a&&(k==n[d]||k==r[d]?(d==0&&(p=v),m=v+1,d++):d=0));let C,b=k<255?k>=48&&k<=57||k>=97&&k<=122?2:k>=65&&k<=90?1:0:(C=sd(k))!=C.toLowerCase()?1:C!=C.toUpperCase()?2:0;(!v||b==1&&w||x==0&&b!=0)&&(n[h]==k||r[h]==k&&(f=!0)?o[h++]=v:o.length&&(g=!1)),x=b,v+=fn(k)}return h==a&&o[0]==0&&g?this.result(-100+(f?-200:0),o,e):d==a&&p==0?this.ret(-200-e.length+(m==e.length?0:-100),[0,m]):l>-1?this.ret(-700-e.length,[l,l+this.pattern.length]):d==a?this.ret(-900-e.length,[p,m]):h==a?this.result(-100+(f?-200:0)+-700+(g?0:-1100),o,e):n.length==2?null:this.result((i[0]?-700:0)+-200+-1100,i,e)}result(e,n,r){let i=[],s=0;for(let o of n){let l=o+(this.astral?fn(ft(r,o)):1);s&&i[s-1]==o?i[s-1]=l:(i[s++]=o,i[s++]=l)}return this.ret(e-r.length,i)}}class EA{constructor(e){this.pattern=e,this.matched=[],this.score=0,this.folded=e.toLowerCase()}match(e){if(e.length<this.pattern.length)return null;let n=e.slice(0,this.pattern.length),r=n==this.pattern?0:n.toLowerCase()==this.folded?-200:null;return r==null?null:(this.matched=[0,n.length],this.score=r+(e.length==this.pattern.length?0:-100),this)}}const _e=z.define({combine(t){return Zt(t,{activateOnTyping:!0,activateOnCompletion:()=>!1,activateOnTypingDelay:100,selectOnOpen:!0,override:null,closeOnBlur:!0,maxRenderedOptions:100,defaultKeymap:!0,tooltipClass:()=>"",optionClass:()=>"",aboveCursor:!1,icons:!0,addToOptions:[],positionInfo:LA,filterStrict:!1,compareCompletions:(e,n)=>(e.sortText||e.label).localeCompare(n.sortText||n.label),interactionDelay:75,updateSyncTime:100},{defaultKeymap:(e,n)=>e&&n,closeOnBlur:(e,n)=>e&&n,icons:(e,n)=>e&&n,tooltipClass:(e,n)=>r=>Bg(e(r),n(r)),optionClass:(e,n)=>r=>Bg(e(r),n(r)),addToOptions:(e,n)=>e.concat(n),filterStrict:(e,n)=>e||n})}});function Bg(t,e){return t?e?t+" "+e:t:e}function LA(t,e,n,r,i,s){let o=t.textDirection==ue.RTL,l=o,a=!1,c="top",u,h,f=e.left-i.left,d=i.right-e.right,p=r.right-r.left,m=r.bottom-r.top;if(l&&f<Math.min(p,d)?l=!1:!l&&d<Math.min(p,f)&&(l=!0),p<=(l?f:d))u=Math.max(i.top,Math.min(n.top,i.bottom-m))-e.top,h=Math.min(400,l?f:d);else{a=!0,h=Math.min(400,(o?e.right:i.right-e.left)-30);let v=i.bottom-e.bottom;v>=m||v>e.top?u=n.bottom-e.top:(c="bottom",u=e.bottom-n.top)}let w=(e.bottom-e.top)/s.offsetHeight,g=(e.right-e.left)/s.offsetWidth;return{style:`${c}: ${u/w}px; max-width: ${h/g}px`,class:"cm-completionInfo-"+(a?o?"left-narrow":"right-narrow":l?"left":"right")}}const Wd=K.define();function DA(t){let e=t.addToOptions.slice();return t.icons&&e.push({render(n){let r=document.createElement("div");return r.classList.add("cm-completionIcon"),n.type&&r.classList.add(...n.type.split(/\s+/g).map(i=>"cm-completionIcon-"+i)),r.setAttribute("aria-hidden","true"),r},position:20}),e.push({render(n,r,i,s){let o=document.createElement("span");o.className="cm-completionLabel";let l=n.displayLabel||n.label,a=0;for(let c=0;c<s.length;){let u=s[c++],h=s[c++];u>a&&o.appendChild(document.createTextNode(l.slice(a,u)));let f=o.appendChild(document.createElement("span"));f.appendChild(document.createTextNode(l.slice(u,h))),f.className="cm-completionMatchedText",a=h}return a<l.length&&o.appendChild(document.createTextNode(l.slice(a))),o},position:50},{render(n){if(!n.detail)return null;let r=document.createElement("span");return r.className="cm-completionDetail",r.textContent=n.detail,r},position:80}),e.sort((n,r)=>n.position-r.position).map(n=>n.render)}function mu(t,e,n){if(t<=n)return{from:0,to:t};if(e<0&&(e=0),e<=t>>1){let i=Math.floor(e/n);return{from:i*n,to:(i+1)*n}}let r=Math.ceil((t-e)/n);return{from:t-r*n,to:t-(r-1)*n}}class OA{constructor(e,n,r){this.view=e,this.stateField=n,this.applyCompletion=r,this.info=null,this.infoDestroy=null,this.placeInfoReq={read:()=>this.measureInfo(),write:a=>this.placeInfo(a),key:this},this.space=null,this.currentClass="";let i=e.state.field(n),{options:s,selected:o}=i.open,l=e.state.facet(_e);this.optionContent=DA(l),this.optionClass=l.optionClass,this.tooltipClass=l.tooltipClass,this.range=mu(s.length,o,l.maxRenderedOptions),this.dom=document.createElement("div"),this.dom.className="cm-tooltip-autocomplete",this.updateTooltipClass(e.state),this.dom.addEventListener("mousedown",a=>{let{options:c}=e.state.field(n).open;for(let u=a.target,h;u&&u!=this.dom;u=u.parentNode)if(u.nodeName=="LI"&&(h=/-(\d+)$/.exec(u.id))&&+h[1]<c.length){this.applyCompletion(e,c[+h[1]]),a.preventDefault();return}if(a.target==this.list){let u=this.list.classList.contains("cm-completionListIncompleteTop")&&a.clientY<this.list.firstChild.getBoundingClientRect().top?this.range.from-1:this.list.classList.contains("cm-completionListIncompleteBottom")&&a.clientY>this.list.lastChild.getBoundingClientRect().bottom?this.range.to:null;u!=null&&(e.dispatch({effects:Wd.of(u)}),a.preventDefault())}}),this.dom.addEventListener("focusout",a=>{let c=e.state.field(this.stateField,!1);c&&c.tooltip&&e.state.facet(_e).closeOnBlur&&a.relatedTarget!=e.contentDOM&&e.dispatch({effects:wo.of(null)})}),this.showOptions(s,i.id)}mount(){this.updateSel()}showOptions(e,n){this.list&&this.list.remove(),this.list=this.dom.appendChild(this.createListBox(e,n,this.range)),this.list.addEventListener("scroll",()=>{this.info&&this.view.requestMeasure(this.placeInfoReq)})}update(e){var n;let r=e.state.field(this.stateField),i=e.startState.field(this.stateField);if(this.updateTooltipClass(e.state),r!=i){let{options:s,selected:o,disabled:l}=r.open;(!i.open||i.open.options!=s)&&(this.range=mu(s.length,o,e.state.facet(_e).maxRenderedOptions),this.showOptions(s,r.id)),this.updateSel(),l!=((n=i.open)===null||n===void 0?void 0:n.disabled)&&this.dom.classList.toggle("cm-tooltip-autocomplete-disabled",!!l)}}updateTooltipClass(e){let n=this.tooltipClass(e);if(n!=this.currentClass){for(let r of this.currentClass.split(" "))r&&this.dom.classList.remove(r);for(let r of n.split(" "))r&&this.dom.classList.add(r);this.currentClass=n}}positioned(e){this.space=e,this.info&&this.view.requestMeasure(this.placeInfoReq)}updateSel(){let e=this.view.state.field(this.stateField),n=e.open;(n.selected>-1&&n.selected<this.range.from||n.selected>=this.range.to)&&(this.range=mu(n.options.length,n.selected,this.view.state.facet(_e).maxRenderedOptions),this.showOptions(n.options,e.id));let r=this.updateSelectedOption(n.selected);if(r){this.destroyInfo();let{completion:i}=n.options[n.selected],{info:s}=i;if(!s)return;let o=typeof s=="string"?document.createTextNode(s):s(i);if(!o)return;"then"in o?o.then(l=>{l&&this.view.state.field(this.stateField,!1)==e&&this.addInfoPane(l,i)}).catch(l=>st(this.view.state,l,"completion info")):(this.addInfoPane(o,i),r.setAttribute("aria-describedby",this.info.id))}}addInfoPane(e,n){this.destroyInfo();let r=this.info=document.createElement("div");if(r.className="cm-tooltip cm-completionInfo",r.id="cm-completionInfo-"+Math.floor(Math.random()*65535).toString(16),e.nodeType!=null)r.appendChild(e),this.infoDestroy=null;else{let{dom:i,destroy:s}=e;r.appendChild(i),this.infoDestroy=s||null}this.dom.appendChild(r),this.view.requestMeasure(this.placeInfoReq)}updateSelectedOption(e){let n=null;for(let r=this.list.firstChild,i=this.range.from;r;r=r.nextSibling,i++)r.nodeName!="LI"||!r.id?i--:i==e?r.hasAttribute("aria-selected")||(r.setAttribute("aria-selected","true"),n=r):r.hasAttribute("aria-selected")&&(r.removeAttribute("aria-selected"),r.removeAttribute("aria-describedby"));return n&&BA(this.list,n),n}measureInfo(){let e=this.dom.querySelector("[aria-selected]");if(!e||!this.info)return null;let n=this.dom.getBoundingClientRect(),r=this.info.getBoundingClientRect(),i=e.getBoundingClientRect(),s=this.space;if(!s){let o=this.dom.ownerDocument.documentElement;s={left:0,top:0,right:o.clientWidth,bottom:o.clientHeight}}return i.top>Math.min(s.bottom,n.bottom)-10||i.bottom<Math.max(s.top,n.top)+10?null:this.view.state.facet(_e).positionInfo(this.view,n,i,r,s,this.dom)}placeInfo(e){this.info&&(e?(e.style&&(this.info.style.cssText=e.style),this.info.className="cm-tooltip cm-completionInfo "+(e.class||"")):this.info.style.cssText="top: -1e6px")}createListBox(e,n,r){const i=document.createElement("ul");i.id=n,i.setAttribute("role","listbox"),i.setAttribute("aria-expanded","true"),i.setAttribute("aria-label",this.view.state.phrase("Completions")),i.addEventListener("mousedown",o=>{o.target==i&&o.preventDefault()});let s=null;for(let o=r.from;o<r.to;o++){let{completion:l,match:a}=e[o],{section:c}=l;if(c){let f=typeof c=="string"?c:c.name;if(f!=s&&(o>r.from||r.from==0))if(s=f,typeof c!="string"&&c.header)i.appendChild(c.header(c));else{let d=i.appendChild(document.createElement("completion-section"));d.textContent=f}}const u=i.appendChild(document.createElement("li"));u.id=n+"-"+o,u.setAttribute("role","option");let h=this.optionClass(l);h&&(u.className=h);for(let f of this.optionContent){let d=f(l,this.view.state,this.view,a);d&&u.appendChild(d)}}return r.from&&i.classList.add("cm-completionListIncompleteTop"),r.to<e.length&&i.classList.add("cm-completionListIncompleteBottom"),i}destroyInfo(){this.info&&(this.infoDestroy&&this.infoDestroy(),this.info.remove(),this.info=null)}destroy(){this.destroyInfo()}}function NA(t,e){return n=>new OA(n,t,e)}function BA(t,e){let n=t.getBoundingClientRect(),r=e.getBoundingClientRect(),i=n.height/t.offsetHeight;r.top<n.top?t.scrollTop-=(n.top-r.top)/i:r.bottom>n.bottom&&(t.scrollTop+=(r.bottom-n.bottom)/i)}function _g(t){return(t.boost||0)*100+(t.apply?10:0)+(t.info?5:0)+(t.type?1:0)}function _A(t,e){let n=[],r=null,i=null,s=u=>{n.push(u);let{section:h}=u.completion;if(h){r||(r=[]);let f=typeof h=="string"?h:h.name;r.some(d=>d.name==f)||r.push(typeof h=="string"?{name:f}:h)}},o=e.facet(_e);for(let u of t)if(u.hasResult()){let h=u.result.getMatch;if(u.result.filter===!1)for(let f of u.result.options)s(new Og(f,u.source,h?h(f):[],1e9-n.length));else{let f=e.sliceDoc(u.from,u.to),d,p=o.filterStrict?new EA(f):new PA(f);for(let m of u.result.options)if(d=p.match(m.label)){let w=m.displayLabel?h?h(m,d.matched):[]:d.matched,g=d.score+(m.boost||0);if(s(new Og(m,u.source,w,g)),typeof m.section=="object"&&m.section.rank==="dynamic"){let{name:v}=m.section;i||(i=Object.create(null)),i[v]=Math.max(g,i[v]||-1e9)}}}}if(r){let u=Object.create(null),h=0,f=(d,p)=>(d.rank==="dynamic"&&p.rank==="dynamic"?i[p.name]-i[d.name]:0)||(typeof d.rank=="number"?d.rank:1e9)-(typeof p.rank=="number"?p.rank:1e9)||(d.name<p.name?-1:1);for(let d of r.sort(f))h-=1e5,u[d.name]=h;for(let d of n){let{section:p}=d.completion;p&&(d.score+=u[typeof p=="string"?p:p.name])}}let l=[],a=null,c=o.compareCompletions;for(let u of n.sort((h,f)=>f.score-h.score||c(h.completion,f.completion))){let h=u.completion;!a||a.label!=h.label||a.detail!=h.detail||a.type!=null&&h.type!=null&&a.type!=h.type||a.apply!=h.apply||a.boost!=h.boost?l.push(u):_g(u.completion)>_g(a)&&(l[l.length-1]=u),a=u.completion}return l}class ki{constructor(e,n,r,i,s,o){this.options=e,this.attrs=n,this.tooltip=r,this.timestamp=i,this.selected=s,this.disabled=o}setSelected(e,n){return e==this.selected||e>=this.options.length?this:new ki(this.options,qg(n,e),this.tooltip,this.timestamp,e,this.disabled)}static build(e,n,r,i,s,o){if(i&&!o&&e.some(c=>c.isPending))return i.setDisabled();let l=_A(e,n);if(!l.length)return i&&e.some(c=>c.isPending)?i.setDisabled():null;let a=n.facet(_e).selectOnOpen?0:-1;if(i&&i.selected!=a&&i.selected!=-1){let c=i.options[i.selected].completion;for(let u=0;u<l.length;u++)if(l[u].completion==c){a=u;break}}return new ki(l,qg(r,a),{pos:e.reduce((c,u)=>u.hasResult()?Math.min(c,u.from):c,1e8),create:VA,above:s.aboveCursor},i?i.timestamp:Date.now(),a,!1)}map(e){return new ki(this.options,this.attrs,{...this.tooltip,pos:e.mapPos(this.tooltip.pos)},this.timestamp,this.selected,this.disabled)}setDisabled(){return new ki(this.options,this.attrs,this.tooltip,this.timestamp,this.selected,!0)}}class Oa{constructor(e,n,r){this.active=e,this.id=n,this.open=r}static start(){return new Oa(zA,"cm-ac-"+Math.floor(Math.random()*2e6).toString(36),null)}update(e){let{state:n}=e,r=n.facet(_e),s=(r.override||n.languageDataAt("autocomplete",Wr(n)).map(AA)).map(a=>(this.active.find(u=>u.source==a)||new Bt(a,this.active.some(u=>u.state!=0)?1:0)).update(e,r));s.length==this.active.length&&s.every((a,c)=>a==this.active[c])&&(s=this.active);let o=this.open,l=e.effects.some(a=>a.is(Fd));o&&e.docChanged&&(o=o.map(e.changes)),e.selection||s.some(a=>a.hasResult()&&e.changes.touchesRange(a.from,a.to))||!qA(s,this.active)||l?o=ki.build(s,n,this.id,o,r,l):o&&o.disabled&&!s.some(a=>a.isPending)&&(o=null),!o&&s.every(a=>!a.isPending)&&s.some(a=>a.hasResult())&&(s=s.map(a=>a.hasResult()?new Bt(a.source,0):a));for(let a of e.effects)a.is(Wd)&&(o=o&&o.setSelected(a.value,this.id));return s==this.active&&o==this.open?this:new Oa(s,this.id,o)}get tooltip(){return this.open?this.open.tooltip:null}get attrs(){return this.open?this.open.attrs:this.active.length?WA:FA}}function qA(t,e){if(t==e)return!0;for(let n=0,r=0;;){for(;n<t.length&&!t[n].hasResult();)n++;for(;r<e.length&&!e[r].hasResult();)r++;let i=n==t.length,s=r==e.length;if(i||s)return i==s;if(t[n++].result!=e[r++].result)return!1}}const WA={"aria-autocomplete":"list"},FA={};function qg(t,e){let n={"aria-autocomplete":"list","aria-haspopup":"listbox","aria-controls":t};return e>-1&&(n["aria-activedescendant"]=t+"-"+e),n}const zA=[];function tS(t,e){if(t.isUserEvent("input.complete")){let r=t.annotation(eS);if(r&&e.activateOnCompletion(r))return 12}let n=t.isUserEvent("input.type");return n&&e.activateOnTyping?5:n?1:t.isUserEvent("delete.backward")?2:t.selection?8:t.docChanged?16:0}class Bt{constructor(e,n,r=!1){this.source=e,this.state=n,this.explicit=r}hasResult(){return!1}get isPending(){return this.state==1}update(e,n){let r=tS(e,n),i=this;(r&8||r&16&&this.touches(e))&&(i=new Bt(i.source,0)),r&4&&i.state==0&&(i=new Bt(this.source,1)),i=i.updateFor(e,r);for(let s of e.effects)if(s.is(Da))i=new Bt(i.source,1,s.value);else if(s.is(wo))i=new Bt(i.source,0);else if(s.is(Fd))for(let o of s.value)o.source==i.source&&(i=o);return i}updateFor(e,n){return this.map(e.changes)}map(e){return this}touches(e){return e.changes.touchesRange(Wr(e.state))}}class Di extends Bt{constructor(e,n,r,i,s,o){super(e,3,n),this.limit=r,this.result=i,this.from=s,this.to=o}hasResult(){return!0}updateFor(e,n){var r;if(!(n&3))return this.map(e.changes);let i=this.result;i.map&&!e.changes.empty&&(i=i.map(i,e.changes));let s=e.changes.mapPos(this.from),o=e.changes.mapPos(this.to,1),l=Wr(e.state);if(l>o||!i||n&2&&(Wr(e.startState)==this.from||l<this.limit))return new Bt(this.source,n&4?1:0);let a=e.changes.mapPos(this.limit);return jA(i.validFor,e.state,s,o)?new Di(this.source,this.explicit,a,i,s,o):i.update&&(i=i.update(i,s,o,new J1(e.state,l,!1)))?new Di(this.source,this.explicit,a,i,i.from,(r=i.to)!==null&&r!==void 0?r:Wr(e.state)):new Bt(this.source,1,this.explicit)}map(e){if(e.empty)return this;let n=this.result.map?this.result.map(this.result,e):this.result;return n?new Di(this.source,this.explicit,e.mapPos(this.limit),n,e.mapPos(this.from),e.mapPos(this.to,1)):new Bt(this.source,0)}touches(e){return e.changes.touchesRange(this.from,this.to)}}function jA(t,e,n,r){if(!t)return!1;let i=e.sliceDoc(n,r);return typeof t=="function"?t(i,n,r,e):Z1(t,!0).test(i)}const Fd=K.define({map(t,e){return t.map(n=>n.map(e))}}),dt=ze.define({create(){return Oa.start()},update(t,e){return t.update(e)},provide:t=>[ic.from(t,e=>e.tooltip),F.contentAttributes.from(t,e=>e.attrs)]});function zd(t,e){const n=e.completion.apply||e.completion.label;let r=t.state.field(dt).active.find(i=>i.source==e.source);return r instanceof Di?(typeof n=="string"?t.dispatch({...IA(t.state,n,r.from,r.to),annotations:eS.of(e.completion)}):n(t,e.completion,r.from,r.to),!0):!1}const VA=NA(dt,zd);function wl(t,e="option"){return n=>{let r=n.state.field(dt,!1);if(!r||!r.open||r.open.disabled||Date.now()-r.open.timestamp<n.state.facet(_e).interactionDelay)return!1;let i=1,s;e=="page"&&(s=Pw(n,r.open.tooltip))&&(i=Math.max(2,Math.floor(s.dom.offsetHeight/s.dom.querySelector("li").offsetHeight)-1));let{length:o}=r.open.options,l=r.open.selected>-1?r.open.selected+i*(t?1:-1):t?0:o-1;return l<0?l=e=="page"?0:o-1:l>=o&&(l=e=="page"?o-1:0),n.dispatch({effects:Wd.of(l)}),!0}}const HA=t=>{let e=t.state.field(dt,!1);return t.state.readOnly||!e||!e.open||e.open.selected<0||e.open.disabled||Date.now()-e.open.timestamp<t.state.facet(_e).interactionDelay?!1:zd(t,e.open.options[e.open.selected])},gu=t=>t.state.field(dt,!1)?(t.dispatch({effects:Da.of(!0)}),!0):!1,UA=t=>{let e=t.state.field(dt,!1);return!e||!e.active.some(n=>n.state!=0)?!1:(t.dispatch({effects:wo.of(null)}),!0)};class $A{constructor(e,n){this.active=e,this.context=n,this.time=Date.now(),this.updates=[],this.done=void 0}}const GA=50,KA=1e3,QA=Me.fromClass(class{constructor(t){this.view=t,this.debounceUpdate=-1,this.running=[],this.debounceAccept=-1,this.pendingStart=!1,this.composing=0;for(let e of t.state.field(dt).active)e.isPending&&this.startQuery(e)}update(t){let e=t.state.field(dt),n=t.state.facet(_e);if(!t.selectionSet&&!t.docChanged&&t.startState.field(dt)==e)return;let r=t.transactions.some(s=>{let o=tS(s,n);return o&8||(s.selection||s.docChanged)&&!(o&3)});for(let s=0;s<this.running.length;s++){let o=this.running[s];if(r||o.context.abortOnDocChange&&t.docChanged||o.updates.length+t.transactions.length>GA&&Date.now()-o.time>KA){for(let l of o.context.abortListeners)try{l()}catch(a){st(this.view.state,a)}o.context.abortListeners=null,this.running.splice(s--,1)}else o.updates.push(...t.transactions)}this.debounceUpdate>-1&&clearTimeout(this.debounceUpdate),t.transactions.some(s=>s.effects.some(o=>o.is(Da)))&&(this.pendingStart=!0);let i=this.pendingStart?50:n.activateOnTypingDelay;if(this.debounceUpdate=e.active.some(s=>s.isPending&&!this.running.some(o=>o.active.source==s.source))?setTimeout(()=>this.startUpdate(),i):-1,this.composing!=0)for(let s of t.transactions)s.isUserEvent("input.type")?this.composing=2:this.composing==2&&s.selection&&(this.composing=3)}startUpdate(){this.debounceUpdate=-1,this.pendingStart=!1;let{state:t}=this.view,e=t.field(dt);for(let n of e.active)n.isPending&&!this.running.some(r=>r.active.source==n.source)&&this.startQuery(n);this.running.length&&e.open&&e.open.disabled&&(this.debounceAccept=setTimeout(()=>this.accept(),this.view.state.facet(_e).updateSyncTime))}startQuery(t){let{state:e}=this.view,n=Wr(e),r=new J1(e,n,t.explicit,this.view),i=new $A(t,r);this.running.push(i),Promise.resolve(t.source(r)).then(s=>{i.context.aborted||(i.done=s||null,this.scheduleAccept())},s=>{this.view.dispatch({effects:wo.of(null)}),st(this.view.state,s)})}scheduleAccept(){this.running.every(t=>t.done!==void 0)?this.accept():this.debounceAccept<0&&(this.debounceAccept=setTimeout(()=>this.accept(),this.view.state.facet(_e).updateSyncTime))}accept(){var t;this.debounceAccept>-1&&clearTimeout(this.debounceAccept),this.debounceAccept=-1;let e=[],n=this.view.state.facet(_e),r=this.view.state.field(dt);for(let i=0;i<this.running.length;i++){let s=this.running[i];if(s.done===void 0)continue;if(this.running.splice(i--,1),s.done){let l=Wr(s.updates.length?s.updates[0].startState:this.view.state),a=Math.min(l,s.done.from+(s.active.explicit?0:1)),c=new Di(s.active.source,s.active.explicit,a,s.done,s.done.from,(t=s.done.to)!==null&&t!==void 0?t:l);for(let u of s.updates)c=c.update(u,n);if(c.hasResult()){e.push(c);continue}}let o=r.active.find(l=>l.source==s.active.source);if(o&&o.isPending)if(s.done==null){let l=new Bt(s.active.source,0);for(let a of s.updates)l=l.update(a,n);l.isPending||e.push(l)}else this.startQuery(o)}(e.length||r.open&&r.open.disabled)&&this.view.dispatch({effects:Fd.of(e)})}},{eventHandlers:{blur(t){let e=this.view.state.field(dt,!1);if(e&&e.tooltip&&this.view.state.facet(_e).closeOnBlur){let n=e.open&&Pw(this.view,e.open.tooltip);(!n||!n.dom.contains(t.relatedTarget))&&setTimeout(()=>this.view.dispatch({effects:wo.of(null)}),10)}},compositionstart(){this.composing=1},compositionend(){this.composing==3&&setTimeout(()=>this.view.dispatch({effects:Da.of(!1)}),20),this.composing=0}}}),YA=typeof navigator=="object"&&/Win/.test(navigator.platform),XA=Zr.highest(F.domEventHandlers({keydown(t,e){let n=e.state.field(dt,!1);if(!n||!n.open||n.open.disabled||n.open.selected<0||t.key.length>1||t.ctrlKey&&!(YA&&t.altKey)||t.metaKey)return!1;let r=n.open.options[n.open.selected],i=n.active.find(o=>o.source==r.source),s=r.completion.commitCharacters||i.result.commitCharacters;return s&&s.indexOf(t.key)>-1&&zd(e,r),!1}})),JA=F.baseTheme({".cm-tooltip.cm-tooltip-autocomplete":{"& > ul":{fontFamily:"monospace",whiteSpace:"nowrap",overflow:"hidden auto",maxWidth_fallback:"700px",maxWidth:"min(700px, 95vw)",minWidth:"250px",maxHeight:"10em",height:"100%",listStyle:"none",margin:0,padding:0,"& > li, & > completion-section":{padding:"1px 3px",lineHeight:1.2},"& > li":{overflowX:"hidden",textOverflow:"ellipsis",cursor:"pointer"},"& > completion-section":{display:"list-item",borderBottom:"1px solid silver",paddingLeft:"0.5em",opacity:.7}}},"&light .cm-tooltip-autocomplete ul li[aria-selected]":{background:"#17c",color:"white"},"&light .cm-tooltip-autocomplete-disabled ul li[aria-selected]":{background:"#777"},"&dark .cm-tooltip-autocomplete ul li[aria-selected]":{background:"#347",color:"white"},"&dark .cm-tooltip-autocomplete-disabled ul li[aria-selected]":{background:"#444"},".cm-completionListIncompleteTop:before, .cm-completionListIncompleteBottom:after":{content:'"···"',opacity:.5,display:"block",textAlign:"center",cursor:"pointer"},".cm-tooltip.cm-completionInfo":{position:"absolute",padding:"3px 9px",width:"max-content",maxWidth:"400px",boxSizing:"border-box",whiteSpace:"pre-line"},".cm-completionInfo.cm-completionInfo-left":{right:"100%"},".cm-completionInfo.cm-completionInfo-right":{left:"100%"},".cm-completionInfo.cm-completionInfo-left-narrow":{right:"30px"},".cm-completionInfo.cm-completionInfo-right-narrow":{left:"30px"},"&light .cm-snippetField":{backgroundColor:"#00000022"},"&dark .cm-snippetField":{backgroundColor:"#ffffff22"},".cm-snippetFieldPosition":{verticalAlign:"text-top",width:0,height:"1.15em",display:"inline-block",margin:"0 -0.7px -.7em",borderLeft:"1.4px dotted #888"},".cm-completionMatchedText":{textDecoration:"underline"},".cm-completionDetail":{marginLeft:"0.5em",fontStyle:"italic"},".cm-completionIcon":{fontSize:"90%",width:".8em",display:"inline-block",textAlign:"center",paddingRight:".6em",opacity:"0.6",boxSizing:"content-box"},".cm-completionIcon-function, .cm-completionIcon-method":{"&:after":{content:"'ƒ'"}},".cm-completionIcon-class":{"&:after":{content:"'○'"}},".cm-completionIcon-interface":{"&:after":{content:"'◌'"}},".cm-completionIcon-variable":{"&:after":{content:"'𝑥'"}},".cm-completionIcon-constant":{"&:after":{content:"'𝐶'"}},".cm-completionIcon-type":{"&:after":{content:"'𝑡'"}},".cm-completionIcon-enum":{"&:after":{content:"'∪'"}},".cm-completionIcon-property":{"&:after":{content:"'□'"}},".cm-completionIcon-keyword":{"&:after":{content:"'🔑︎'"}},".cm-completionIcon-namespace":{"&:after":{content:"'▢'"}},".cm-completionIcon-text":{"&:after":{content:"'abc'",fontSize:"50%",verticalAlign:"middle"}}}),So={brackets:["(","[","{","'",'"'],before:")]}:;>",stringPrefixes:[]},Or=K.define({map(t,e){let n=e.mapPos(t,-1,it.TrackAfter);return n??void 0}}),jd=new class extends ur{};jd.startSide=1;jd.endSide=-1;const nS=ze.define({create(){return X.empty},update(t,e){if(t=t.map(e.changes),e.selection){let n=e.state.doc.lineAt(e.selection.main.head);t=t.update({filter:r=>r>=n.from&&r<=n.to})}for(let n of e.effects)n.is(Or)&&(t=t.update({add:[jd.range(n.value,n.value+1)]}));return t}});function ZA(){return[tP,nS]}const vu="()[]{}<>«»»«［］｛｝";function rS(t){for(let e=0;e<vu.length;e+=2)if(vu.charCodeAt(e)==t)return vu.charAt(e+1);return sd(t<128?t:t+1)}function iS(t,e){return t.languageDataAt("closeBrackets",e)[0]||So}const eP=typeof navigator=="object"&&/Android\b/.test(navigator.userAgent),tP=F.inputHandler.of((t,e,n,r)=>{if((eP?t.composing:t.compositionStarted)||t.state.readOnly)return!1;let i=t.state.selection.main;if(r.length>2||r.length==2&&fn(ft(r,0))==1||e!=i.from||n!=i.to)return!1;let s=iP(t.state,r);return s?(t.dispatch(s),!0):!1}),nP=({state:t,dispatch:e})=>{if(t.readOnly)return!1;let r=iS(t,t.selection.main.head).brackets||So.brackets,i=null,s=t.changeByRange(o=>{if(o.empty){let l=sP(t.doc,o.head);for(let a of r)if(a==l&&dc(t.doc,o.head)==rS(ft(a,0)))return{changes:{from:o.head-a.length,to:o.head+a.length},range:R.cursor(o.head-a.length)}}return{range:i=o}});return i||e(t.update(s,{scrollIntoView:!0,userEvent:"delete.backward"})),!i},rP=[{key:"Backspace",run:nP}];function iP(t,e){let n=iS(t,t.selection.main.head),r=n.brackets||So.brackets;for(let i of r){let s=rS(ft(i,0));if(e==i)return s==i?aP(t,i,r.indexOf(i+i+i)>-1,n):oP(t,i,s,n.before||So.before);if(e==s&&sS(t,t.selection.main.from))return lP(t,i,s)}return null}function sS(t,e){let n=!1;return t.field(nS).between(0,t.doc.length,r=>{r==e&&(n=!0)}),n}function dc(t,e){let n=t.sliceString(e,e+2);return n.slice(0,fn(ft(n,0)))}function sP(t,e){let n=t.sliceString(e-2,e);return fn(ft(n,0))==n.length?n:n.slice(1)}function oP(t,e,n,r){let i=null,s=t.changeByRange(o=>{if(!o.empty)return{changes:[{insert:e,from:o.from},{insert:n,from:o.to}],effects:Or.of(o.to+e.length),range:R.range(o.anchor+e.length,o.head+e.length)};let l=dc(t.doc,o.head);return!l||/\s/.test(l)||r.indexOf(l)>-1?{changes:{insert:e+n,from:o.head},effects:Or.of(o.head+e.length),range:R.cursor(o.head+e.length)}:{range:i=o}});return i?null:t.update(s,{scrollIntoView:!0,userEvent:"input.type"})}function lP(t,e,n){let r=null,i=t.changeByRange(s=>s.empty&&dc(t.doc,s.head)==n?{changes:{from:s.head,to:s.head+n.length,insert:n},range:R.cursor(s.head+n.length)}:r={range:s});return r?null:t.update(i,{scrollIntoView:!0,userEvent:"input.type"})}function aP(t,e,n,r){let i=r.stringPrefixes||So.stringPrefixes,s=null,o=t.changeByRange(l=>{if(!l.empty)return{changes:[{insert:e,from:l.from},{insert:e,from:l.to}],effects:Or.of(l.to+e.length),range:R.range(l.anchor+e.length,l.head+e.length)};let a=l.head,c=dc(t.doc,a),u;if(c==e){if(Wg(t,a))return{changes:{insert:e+e,from:a},effects:Or.of(a+e.length),range:R.cursor(a+e.length)};if(sS(t,a)){let f=n&&t.sliceDoc(a,a+e.length*3)==e+e+e?e+e+e:e;return{changes:{from:a,to:a+f.length,insert:f},range:R.cursor(a+f.length)}}}else{if(n&&t.sliceDoc(a-2*e.length,a)==e+e&&(u=Fg(t,a-2*e.length,i))>-1&&Wg(t,u))return{changes:{insert:e+e+e+e,from:a},effects:Or.of(a+e.length),range:R.cursor(a+e.length)};if(t.charCategorizer(a)(c)!=we.Word&&Fg(t,a,i)>-1&&!cP(t,a,e,i))return{changes:{insert:e+e,from:a},effects:Or.of(a+e.length),range:R.cursor(a+e.length)}}return{range:s=l}});return s?null:t.update(o,{scrollIntoView:!0,userEvent:"input.type"})}function Wg(t,e){let n=et(t).resolveInner(e+1);return n.parent&&n.from==e}function cP(t,e,n,r){let i=et(t).resolveInner(e,-1),s=r.reduce((o,l)=>Math.max(o,l.length),0);for(let o=0;o<5;o++){let l=t.sliceDoc(i.from,Math.min(i.to,i.from+n.length+s)),a=l.indexOf(n);if(!a||a>-1&&r.indexOf(l.slice(0,a))>-1){let u=i.firstChild;for(;u&&u.from==i.from&&u.to-u.from>n.length+a;){if(t.sliceDoc(u.to-n.length,u.to)==n)return!1;u=u.firstChild}return!0}let c=i.to==e&&i.parent;if(!c)break;i=c}return!1}function Fg(t,e,n){let r=t.charCategorizer(e);if(r(t.sliceDoc(e-1,e))!=we.Word)return e;for(let i of n){let s=e-i.length;if(t.sliceDoc(s,e)==i&&r(t.sliceDoc(s-1,s))!=we.Word)return s}return-1}function uP(t={}){return[XA,dt,_e.of(t),QA,hP,JA]}const oS=[{key:"Ctrl-Space",run:gu},{mac:"Alt-`",run:gu},{mac:"Alt-i",run:gu},{key:"Escape",run:UA},{key:"ArrowDown",run:wl(!0)},{key:"ArrowUp",run:wl(!1)},{key:"PageDown",run:wl(!0,"page")},{key:"PageUp",run:wl(!1,"page")},{key:"Enter",run:HA}],hP=Zr.highest(rc.computeN([_e],t=>t.facet(_e).defaultKeymap?[oS]:[]));class zg{constructor(e,n,r){this.from=e,this.to=n,this.diagnostic=r}}class Ar{constructor(e,n,r){this.diagnostics=e,this.panel=n,this.selected=r}static init(e,n,r){let i=r.facet(vn).markerFilter;i&&(e=i(e,r));let s=e.slice().sort((d,p)=>d.from-p.from||d.to-p.to),o=new xn,l=[],a=0,c=r.doc.iter(),u=0,h=r.doc.length;for(let d=0;;){let p=d==s.length?null:s[d];if(!p&&!l.length)break;let m,w;if(l.length)m=a,w=l.reduce((y,x)=>Math.min(y,x.to),p&&p.from>m?p.from:1e8);else{if(m=p.from,m>h)break;w=p.to,l.push(p),d++}for(;d<s.length;){let y=s[d];if(y.from==m&&(y.to>y.from||y.to==m))l.push(y),d++,w=Math.min(y.to,w);else{w=Math.min(y.from,w);break}}w=Math.min(w,h);let g=!1;if(l.some(y=>y.from==m&&(y.to==w||w==h))&&(g=m==w,!g&&w-m<10)){let y=m-(u+c.value.length);y>0&&(c.next(y),u=m);for(let x=m;;){if(x>=w){g=!0;break}if(!c.lineBreak&&u+c.value.length>x)break;x=u+c.value.length,u+=c.value.length,c.next()}}let v=pS(l);if(g)o.add(m,m,U.widget({widget:new SP(v),diagnostics:l.slice()}));else{let y=l.reduce((x,k)=>k.markClass?x+" "+k.markClass:x,"");o.add(m,w,U.mark({class:"cm-lintRange cm-lintRange-"+v+y,diagnostics:l.slice(),inclusiveEnd:l.some(x=>x.to>w)}))}if(a=w,a==h)break;for(let y=0;y<l.length;y++)l[y].to<=a&&l.splice(y--,1)}let f=o.finish();return new Ar(f,n,gr(f))}}function gr(t,e=null,n=0){let r=null;return t.between(n,1e9,(i,s,{spec:o})=>{if(!(e&&o.diagnostics.indexOf(e)<0))if(!r)r=new zg(i,s,e||o.diagnostics[0]);else{if(o.diagnostics.indexOf(r.diagnostic)<0)return!1;r=new zg(r.from,s,r.diagnostic)}}),r}function lS(t,e){let n=e.pos,r=e.end||n,i=t.state.facet(vn).hideOn(t,n,r);if(i!=null)return i;let s=t.startState.doc.lineAt(e.pos);return!!(t.effects.some(o=>o.is(pc))||t.changes.touchesRange(s.from,Math.max(s.to,r)))}function aS(t,e){return t.field(Tt,!1)?e:e.concat(K.appendConfig.of(yS))}function cS(t,e){return{effects:aS(t,[pc.of(e)])}}const pc=K.define(),Vd=K.define(),uS=K.define(),Tt=ze.define({create(){return new Ar(U.none,null,null)},update(t,e){if(e.docChanged&&t.diagnostics.size){let n=t.diagnostics.map(e.changes),r=null,i=t.panel;if(t.selected){let s=e.changes.mapPos(t.selected.from,1);r=gr(n,t.selected.diagnostic,s)||gr(n,null,s)}!n.size&&i&&e.state.facet(vn).autoPanel&&(i=null),t=new Ar(n,i,r)}for(let n of e.effects)if(n.is(pc)){let r=e.state.facet(vn).autoPanel?n.value.length?ko.open:null:t.panel;t=Ar.init(n.value,r,e.state)}else n.is(Vd)?t=new Ar(t.diagnostics,n.value?ko.open:null,t.selected):n.is(uS)&&(t=new Ar(t.diagnostics,t.panel,n.value));return t},provide:t=>[fo.from(t,e=>e.panel),F.decorations.from(t,e=>e.diagnostics)]}),fP=U.mark({class:"cm-lintRange cm-lintRange-active"});function dP(t,e,n){let{diagnostics:r}=t.state.field(Tt),i,s=-1,o=-1;r.between(e-(n<0?1:0),e+(n>0?1:0),(a,c,{spec:u})=>{if(e>=a&&e<=c&&(a==c||(e>a||n>0)&&(e<c||n<0)))return i=u.diagnostics,s=a,o=c,!1});let l=t.state.facet(vn).tooltipFilter;return i&&l&&(i=l(i,t.state)),i?{pos:s,end:o,above:!0,create(){return{dom:hS(t,i)}}}:null}function hS(t,e){return le("ul",{class:"cm-tooltip-lint"},e.map(n=>dS(t,n,!1)))}const pP=t=>{let e=t.state.field(Tt,!1);(!e||!e.panel)&&t.dispatch({effects:aS(t.state,[Vd.of(!0)])});let n=oc(t,ko.open);return n&&n.dom.querySelector(".cm-panel-lint ul").focus(),!0},jg=t=>{let e=t.state.field(Tt,!1);return!e||!e.panel?!1:(t.dispatch({effects:Vd.of(!1)}),!0)},mP=t=>{let e=t.state.field(Tt,!1);if(!e)return!1;let n=t.state.selection.main,r=gr(e.diagnostics,null,n.to+1);return!r&&(r=gr(e.diagnostics,null,0),!r||r.from==n.from&&r.to==n.to)?!1:(t.dispatch({selection:{anchor:r.from,head:r.to},scrollIntoView:!0}),cM(t,r.from,1,{tooltip:vS,until:i=>i.docChanged||i.newSelection.main.head<r.from||i.newSelection.main.head>r.to}),!0)},gP=[{key:"Mod-Shift-m",run:pP,preventDefault:!0},{key:"F8",run:mP}],vP=Me.fromClass(class{constructor(t){this.view=t,this.timeout=-1,this.set=!0;let{delay:e}=t.state.facet(vn);this.lintTime=Date.now()+e,this.run=this.run.bind(this),this.timeout=setTimeout(this.run,e)}run(){clearTimeout(this.timeout);let t=Date.now();if(t<this.lintTime-10)this.timeout=setTimeout(this.run,this.lintTime-t);else{this.set=!1;let{state:e}=this.view,{sources:n}=e.facet(vn);n.length&&yP(n.map(r=>Promise.resolve(r(this.view))),r=>{this.view.state.doc==e.doc&&this.view.dispatch(cS(this.view.state,r.reduce((i,s)=>i.concat(s))))},r=>{st(this.view.state,r)})}}update(t){let e=t.state.facet(vn);(t.docChanged||e!=t.startState.facet(vn)||e.needsRefresh&&e.needsRefresh(t))&&(this.lintTime=Date.now()+e.delay,this.set||(this.set=!0,this.timeout=setTimeout(this.run,e.delay)))}force(){this.set&&(this.lintTime=Date.now(),this.run())}destroy(){clearTimeout(this.timeout)}});function yP(t,e,n){let r=[],i=-1;for(let s of t)s.then(o=>{r.push(o),clearTimeout(i),r.length==t.length?e(r):i=setTimeout(()=>e(r),200)},n)}const vn=z.define({combine(t){return{sources:t.map(e=>e.source).filter(e=>e!=null),...Zt(t.map(e=>e.config),{delay:750,markerFilter:null,tooltipFilter:null,needsRefresh:null,hideOn:()=>null},{delay:Math.max,markerFilter:Vg,tooltipFilter:Vg,needsRefresh:(e,n)=>e?n?r=>e(r)||n(r):e:n,hideOn:(e,n)=>e?n?(r,i,s)=>e(r,i,s)||n(r,i,s):e:n,autoPanel:(e,n)=>e||n})}}});function Vg(t,e){return t?e?(n,r)=>e(t(n,r),r):t:e}function wP(t,e={}){return[vn.of({source:t,config:e}),vP,yS]}function fS(t){let e=[];if(t)e:for(let{name:n}of t){for(let r=0;r<n.length;r++){let i=n[r];if(/[a-zA-Z]/.test(i)&&!e.some(s=>s.toLowerCase()==i.toLowerCase())){e.push(i);continue e}}e.push("")}return e}function dS(t,e,n){var r;let i=n?fS(e.actions):[];return le("li",{class:"cm-diagnostic cm-diagnostic-"+e.severity},le("span",{class:"cm-diagnosticText"},e.renderMessage?e.renderMessage(t):e.message),(r=e.actions)===null||r===void 0?void 0:r.map((s,o)=>{let l=!1,a=d=>{if(d.preventDefault(),l)return;l=!0;let p=gr(t.state.field(Tt).diagnostics,e);p&&s.apply(t,p.from,p.to)},{name:c}=s,u=i[o]?c.indexOf(i[o]):-1,h=u<0?c:[c.slice(0,u),le("u",c.slice(u,u+1)),c.slice(u+1)],f=s.markClass?" "+s.markClass:"";return le("button",{type:"button",class:"cm-diagnosticAction"+f,onclick:a,onmousedown:a,"aria-label":` Action: ${c}${u<0?"":` (access key "${i[o]})"`}.`},h)}),e.source&&le("div",{class:"cm-diagnosticSource"},e.source))}class SP extends qn{constructor(e){super(),this.sev=e}eq(e){return e.sev==this.sev}toDOM(){return le("span",{class:"cm-lintPoint cm-lintPoint-"+this.sev})}}class Hg{constructor(e,n){this.diagnostic=n,this.id="item_"+Math.floor(Math.random()*4294967295).toString(16),this.dom=dS(e,n,!0),this.dom.id=this.id,this.dom.setAttribute("role","option")}}class ko{constructor(e){this.view=e,this.items=[];let n=i=>{if(!(i.ctrlKey||i.altKey||i.metaKey)){if(i.keyCode==27)jg(this.view),this.view.focus();else if(i.keyCode==38||i.keyCode==33)this.moveSelection((this.selectedIndex-1+this.items.length)%this.items.length);else if(i.keyCode==40||i.keyCode==34)this.moveSelection((this.selectedIndex+1)%this.items.length);else if(i.keyCode==36)this.moveSelection(0);else if(i.keyCode==35)this.moveSelection(this.items.length-1);else if(i.keyCode==13)this.view.focus();else if(i.keyCode>=65&&i.keyCode<=90&&this.selectedIndex>=0){let{diagnostic:s}=this.items[this.selectedIndex],o=fS(s.actions);for(let l=0;l<o.length;l++)if(o[l].toUpperCase().charCodeAt(0)==i.keyCode){let a=gr(this.view.state.field(Tt).diagnostics,s);a&&s.actions[l].apply(e,a.from,a.to)}}else return;i.preventDefault()}},r=i=>{for(let s=0;s<this.items.length;s++)this.items[s].dom.contains(i.target)&&this.moveSelection(s)};this.list=le("ul",{tabIndex:0,role:"listbox","aria-label":this.view.state.phrase("Diagnostics"),onkeydown:n,onclick:r}),this.dom=le("div",{class:"cm-panel-lint"},this.list,le("button",{type:"button",name:"close","aria-label":this.view.state.phrase("close"),onclick:()=>jg(this.view)},"×")),this.update()}get selectedIndex(){let e=this.view.state.field(Tt).selected;if(!e)return-1;for(let n=0;n<this.items.length;n++)if(this.items[n].diagnostic==e.diagnostic)return n;return-1}update(){let{diagnostics:e,selected:n}=this.view.state.field(Tt),r=0,i=!1,s=null,o=new Set;for(e.between(0,this.view.state.doc.length,(l,a,{spec:c})=>{for(let u of c.diagnostics){if(o.has(u))continue;o.add(u);let h=-1,f;for(let d=r;d<this.items.length;d++)if(this.items[d].diagnostic==u){h=d;break}h<0?(f=new Hg(this.view,u),this.items.splice(r,0,f),i=!0):(f=this.items[h],h>r&&(this.items.splice(r,h-r),i=!0)),n&&f.diagnostic==n.diagnostic?f.dom.hasAttribute("aria-selected")||(f.dom.setAttribute("aria-selected","true"),s=f):f.dom.hasAttribute("aria-selected")&&f.dom.removeAttribute("aria-selected"),r++}});r<this.items.length&&!(this.items.length==1&&this.items[0].diagnostic.from<0);)i=!0,this.items.pop();this.items.length==0&&(this.items.push(new Hg(this.view,{from:-1,to:-1,severity:"info",message:this.view.state.phrase("No diagnostics")})),i=!0),s?(this.list.setAttribute("aria-activedescendant",s.id),this.view.requestMeasure({key:this,read:()=>({sel:s.dom.getBoundingClientRect(),panel:this.list.getBoundingClientRect()}),write:({sel:l,panel:a})=>{let c=a.height/this.list.offsetHeight;l.top<a.top?this.list.scrollTop-=(a.top-l.top)/c:l.bottom>a.bottom&&(this.list.scrollTop+=(l.bottom-a.bottom)/c)}})):this.selectedIndex<0&&this.list.removeAttribute("aria-activedescendant"),i&&this.sync()}sync(){let e=this.list.firstChild;function n(){let r=e;e=r.nextSibling,r.remove()}for(let r of this.items)if(r.dom.parentNode==this.list){for(;e!=r.dom;)n();e=r.dom.nextSibling}else this.list.insertBefore(r.dom,e);for(;e;)n()}moveSelection(e){if(this.selectedIndex<0)return;let n=this.view.state.field(Tt),r=gr(n.diagnostics,this.items[e].diagnostic);r&&this.view.dispatch({selection:{anchor:r.from,head:r.to},scrollIntoView:!0,effects:uS.of(r)})}static open(e){return new ko(e)}}function jl(t,e='viewBox="0 0 40 40"'){return`url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" ${e}>${encodeURIComponent(t)}</svg>')`}function Sl(t){return jl(`<path d="m0 2.5 l2 -1.5 l1 0 l2 1.5 l1 0" stroke="${t}" fill="none" stroke-width=".7"/>`,'width="6" height="3"')}const kP=F.baseTheme({".cm-diagnostic":{padding:"3px 6px 3px 8px",marginLeft:"-1px",display:"block",whiteSpace:"pre-wrap"},".cm-diagnostic-error":{borderLeft:"5px solid #d11"},".cm-diagnostic-warning":{borderLeft:"5px solid orange"},".cm-diagnostic-info":{borderLeft:"5px solid #999"},".cm-diagnostic-hint":{borderLeft:"5px solid #66d"},".cm-diagnosticAction":{font:"inherit",border:"none",padding:"2px 4px",backgroundColor:"#444",color:"white",borderRadius:"3px",marginLeft:"8px",cursor:"pointer"},".cm-diagnosticSource":{fontSize:"70%",opacity:.7},".cm-lintRange":{backgroundPosition:"left bottom",backgroundRepeat:"repeat-x",paddingBottom:"0.7px"},".cm-lintRange-error":{backgroundImage:Sl("#f11")},".cm-lintRange-warning":{backgroundImage:Sl("orange")},".cm-lintRange-info":{backgroundImage:Sl("#999")},".cm-lintRange-hint":{backgroundImage:Sl("#66d")},".cm-lintRange-active":{backgroundColor:"#ffdd9980"},".cm-tooltip-lint":{padding:0,margin:0},".cm-lintPoint":{position:"relative","&:after":{content:'""',position:"absolute",bottom:0,left:"-2px",borderLeft:"3px solid transparent",borderRight:"3px solid transparent",borderBottom:"4px solid #d11"}},".cm-lintPoint-warning":{"&:after":{borderBottomColor:"orange"}},".cm-lintPoint-info":{"&:after":{borderBottomColor:"#999"}},".cm-lintPoint-hint":{"&:after":{borderBottomColor:"#66d"}},".cm-panel.cm-panel-lint":{position:"relative","& ul":{maxHeight:"100px",overflowY:"auto","& [aria-selected]":{backgroundColor:"#ddd","& u":{textDecoration:"underline"}},"&:focus [aria-selected]":{background_fallback:"#bdf",backgroundColor:"Highlight",color_fallback:"white",color:"HighlightText"},"& u":{textDecoration:"none"},padding:0,margin:0},"& [name=close]":{position:"absolute",top:"0",right:"2px",background:"inherit",border:"none",font:"inherit",padding:0,margin:0}},"&dark .cm-lintRange-active":{backgroundColor:"#86714a80"},"&dark .cm-panel.cm-panel-lint ul":{"& [aria-selected]":{backgroundColor:"#2e343e"}}});function xP(t){return t=="error"?4:t=="warning"?3:t=="info"?2:1}function pS(t){let e="hint",n=1;for(let r of t){let i=xP(r.severity);i>n&&(n=i,e=r.severity)}return e}class mS extends Cn{constructor(e){super(),this.diagnostics=e,this.severity=pS(e)}toDOM(e){let n=document.createElement("div");n.className="cm-lint-marker cm-lint-marker-"+this.severity;let r=this.diagnostics,i=e.state.facet(mc).tooltipFilter;return i&&(r=i(r,e.state)),r.length&&(n.onmouseover=()=>bP(e,n,r)),n}}function CP(t,e){let n=r=>{let i=e.getBoundingClientRect();if(!(r.clientX>i.left-10&&r.clientX<i.right+10&&r.clientY>i.top-10&&r.clientY<i.bottom+10)){for(let s=r.target;s;s=s.parentNode)if(s.nodeType==1&&s.classList.contains("cm-tooltip-lint"))return;window.removeEventListener("mousemove",n),t.state.field(gS)&&t.dispatch({effects:Hd.of(null)})}};window.addEventListener("mousemove",n)}function bP(t,e,n){function r(){let o=t.elementAtHeight(e.getBoundingClientRect().top+5-t.documentTop);t.coordsAtPos(o.from)&&t.dispatch({effects:Hd.of({pos:o.from,above:!1,clip:!1,create(){return{dom:hS(t,n),getCoords:()=>e.getBoundingClientRect()}}})}),e.onmouseout=e.onmousemove=null,CP(t,e)}let{hoverTime:i}=t.state.facet(mc),s=setTimeout(r,i);e.onmouseout=()=>{clearTimeout(s),e.onmouseout=e.onmousemove=null},e.onmousemove=()=>{clearTimeout(s),s=setTimeout(r,i)}}function TP(t,e){let n=Object.create(null);for(let i of e){let s=t.lineAt(i.from);(n[s.from]||(n[s.from]=[])).push(i)}let r=[];for(let i in n)r.push(new mS(n[i]).range(+i));return X.of(r,!0)}const RP=Ow({class:"cm-gutter-lint",markers:t=>t.state.field(cf),widgetMarker:(t,e,n)=>{let r=[];return t.state.field(cf).between(n.from,n.to,(i,s,o)=>{i>n.from&&i<n.to&&r.push(...o.diagnostics)}),r.length?new mS(r):null}}),cf=ze.define({create(){return X.empty},update(t,e){t=t.map(e.changes);let n=e.state.facet(mc).markerFilter;for(let r of e.effects)if(r.is(pc)){let i=r.value;n&&(i=n(i||[],e.state)),t=TP(e.state.doc,i.slice(0))}return t}}),Hd=K.define(),gS=ze.define({create(){return null},update(t,e){return t&&e.docChanged&&(t=lS(e,t)?null:{...t,pos:e.changes.mapPos(t.pos)}),e.effects.reduce((n,r)=>r.is(Hd)?r.value:n,t)},provide:t=>ic.from(t)}),MP=F.baseTheme({".cm-gutter-lint":{width:"1.4em","& .cm-gutterElement":{padding:".2em"}},".cm-lint-marker":{width:"1em",height:"1em"},".cm-lint-marker-info":{content:jl('<path fill="#aaf" stroke="#77e" stroke-width="6" stroke-linejoin="round" d="M5 5L35 5L35 35L5 35Z"/>')},".cm-lint-marker-warning":{content:jl('<path fill="#fe8" stroke="#fd7" stroke-width="6" stroke-linejoin="round" d="M20 6L37 35L3 35Z"/>')},".cm-lint-marker-error":{content:jl('<circle cx="20" cy="20" r="15" fill="#f87" stroke="#f43" stroke-width="6"/>')}}),vS=aM(dP,{hideOn:lS}),yS=[Tt,F.decorations.compute([Tt],t=>{let{selected:e,panel:n}=t.field(Tt);return!e||!n||e.from==e.to?U.none:U.set([fP.range(e.from,e.to)])}),vS,kP],mc=z.define({combine(t){return Zt(t,{hoverTime:300,markerFilter:null,tooltipFilter:null})}});function IP(t={}){return[mc.of(t),cf,RP,MP,gS]}const AP=[xM(),TM(),WR(),G2(),g2(),MR(),DR(),ee.allowMultipleSelections.of(!0),n2(),t1(S2,{fallback:!0}),M2(),ZA(),uP(),XR(),eM(),UR(),tA(),rc.of([...rP,...QI,...xA,...nI,...f2,...oS,...gP])],PP=new Set("machine state start hot cold event on do goto entry exit defer ignore var fun return if else while foreach in send raise announce new this halt assert assume print format receive case spec observes test main module union type enum foreign choose sizeof keys values default with as break continue implementation refines compose hidden param interface receives sends creates to assume tuple fairness".split(" ")),EP=new Set("int bool float string machine event seq set map any data duration".split(" ")),LP=new Set("true false null".split(" ")),DP={name:"p",startState:()=>({inComment:!1}),token(t,e){if(e.inComment)return t.skipTo("*/")?(t.next(),t.next(),e.inComment=!1):t.skipToEnd(),"comment";if(t.eatSpace())return null;if(t.match("//"))return t.skipToEnd(),"comment";if(t.match("/*"))return e.inComment=!0,"comment";if(t.match(/^"(?:[^"\\]|\\.)*"?/))return"string";if(t.match(/^\d+(\.\d+)?/))return"number";if(t.match(/^[A-Za-z_]\w*/)){const n=t.current();return PP.has(n)?"keyword":EP.has(n)?"typeName":LP.has(n)?"atom":/^e[A-Z]/.test(n)?"className":/^[A-Z]/.test(n)?"variableName.special":"variableName"}return t.match(/^(==|!=|<=|>=|&&|\|\||\+=|-=|[-+*/%<>=!])/)?"operator":(t.next(),null)},languageData:{commentTokens:{line:"//",block:{open:"/*",close:"*/"}}}},OP=Ad.define(DP),NP=Eo.define([{tag:D.keyword,color:"var(--syn-keyword)",fontWeight:"600"},{tag:D.typeName,color:"var(--syn-type)"},{tag:D.className,color:"var(--syn-event)"},{tag:D.special(D.variableName),color:"var(--syn-name)"},{tag:D.string,color:"var(--syn-string)"},{tag:D.number,color:"var(--syn-number)"},{tag:D.atom,color:"var(--syn-number)"},{tag:D.comment,color:"var(--syn-comment)",fontStyle:"italic"},{tag:D.operator,color:"var(--syn-op)"}]),BP=F.theme({"&":{height:"100%",fontSize:"var(--editor-font, 13px)",backgroundColor:"var(--editor-bg)",color:"var(--fg)"},".cm-scroller":{fontFamily:"var(--mono)",lineHeight:"1.5"},".cm-gutters":{backgroundColor:"var(--editor-bg)",color:"var(--muted)",border:"none"},".cm-activeLine, .cm-activeLineGutter":{backgroundColor:"var(--active-line)"},"&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection":{backgroundColor:"var(--selection) !important"},".cm-cursor":{borderLeftColor:"var(--fg)"},".cm-tooltip":{backgroundColor:"var(--panel)",color:"var(--fg)",border:"1px solid var(--border)"},".cm-flashLine":{backgroundColor:"var(--flash)"},"&.cm-lineWrapping .cm-line":{paddingLeft:"calc(2ch + 6px)",textIndent:"-2ch"}});function _P({path:t,value:e,diagnostics:n,jump:r,onChange:i,onRun:s}){const o=O.useRef(null),l=O.useRef(),a=O.useRef(new Map),c=O.useRef(t),u=O.useRef({onChange:i,onRun:s});u.current={onChange:i,onRun:s};const h=O.useRef([AP,rc.of([{key:"Mod-Enter",run:()=>(u.current.onRun(),!0)},YI]),OP,F.lineWrapping,t1(NP),BP,IP(),wP(null),F.updateListener.of(d=>{d.docChanged&&u.current.onChange(c.current,d.state.doc.toString())})]),f=(d,p)=>{const m=a.current.get(d);return m&&m.doc.toString()===p?m:ee.create({doc:p,extensions:h.current})};return O.useEffect(()=>(l.current=new F({state:f(t,e),parent:o.current}),()=>{var d;return(d=l.current)==null?void 0:d.destroy()}),[]),O.useEffect(()=>{const d=l.current;c.current!==t?(a.current.set(c.current,d.state),c.current=t,d.setState(f(t,e))):d.state.doc.toString()!==e&&d.setState(f(t,e))},[t,e]),O.useEffect(()=>{const d=l.current,p=d.state.doc,m=n.filter(w=>w.line>=1&&w.line<=p.lines).map(w=>{var x;const g=p.line(w.line),v=Math.min(g.from+Math.max(w.col-1,0),g.to),y=((x=/^\w+/.exec(p.sliceString(v,g.to)))==null?void 0:x[0].length)??0;return{from:v,to:Math.max(v+y,v===g.to?v:v+1),severity:"error",message:w.message}});d.dispatch(cS(d.state,m))},[n,t,e]),O.useEffect(()=>{if(!r)return;const d=l.current,p=d.state.doc,m=p.line(Math.min(Math.max(r.line,1),p.lines));d.dispatch({selection:{anchor:m.from},effects:F.scrollIntoView(m.from,{y:"center"})}),d.focus()},[r]),S.jsx("div",{className:"editor",ref:o})}class qP{constructor(){Oo(this,"worker");Oo(this,"nextId",1);Oo(this,"pending",new Map);this.start()}start(){this.worker=new Worker(new URL(""+new URL("p.worker-DcxGMzFh.js",import.meta.url).href,import.meta.url),{type:"module"}),this.worker.onmessage=e=>{const n=this.pending.get(e.data.id);n&&(this.pending.delete(e.data.id),e.data.error?n.reject(new Error(e.data.error)):n.resolve(e.data.result))}}send(e){const n=this.nextId++;return new Promise((r,i)=>{this.pending.set(n,{resolve:r,reject:i}),this.worker.postMessage({id:n,req:e})})}compile(e){return this.send({kind:"compile",files:e})}check(e){return this.send({kind:"check",...e})}cancel(){this.worker.terminate();for(const e of this.pending.values())e.reject(new Error("cancelled"));this.pending.clear(),this.start()}}function Ug(t,e){var i;const n=t[e],r=n.peer!==void 0?(i=t[n.peer])==null?void 0:i.lane:void 0;return[n.lane,n.to,r].filter(s=>s!==void 0)}function WP(t,e){const n=t.steps.map((s,o)=>o);if(!e.size)return{rows:n,cols:t.lanes.map((s,o)=>o)};const r=n.filter(s=>{const o=t.steps[s];return o.kind==="error"||o.kind==="print"||Ug(t.steps,s).some(l=>e.has(l))}),i=new Set(e);for(const s of r)for(const o of Ug(t.steps,s))i.add(o);return{rows:r,cols:t.lanes.map((s,o)=>o).filter(s=>i.has(s))}}const me=t=>typeof t=="string"?t:void 0;function FP(t){var d;const e=JSON.parse(t),n=[],r=new Map,i=[],s=new Map;let o,l;const a=(p,m=!1)=>{if(!p)return;let w=r.get(p);return w===void 0&&(w=n.length,n.push({id:p,label:p,monitor:m}),r.set(p,w)),w},c=p=>JSON.stringify(p??null);for(const{type:p,details:m}of e){const w=m.log??p,g=m.payload==="null"?void 0:m.payload;switch(p){case"CreateMonitor":a(me(m.monitor),!0);break;case"CreateStateMachine":{const v=a(me(m.id)),y=r.get(me(m.creatorName)??"");i.push({kind:"create",lane:y??v,to:v,label:"new",log:w,payload:g});break}case"StateTransition":case"MonitorStateTransition":m.isEntry&&i.push({kind:"state",lane:a(me(m.id)??me(m.monitor),p==="MonitorStateTransition"),label:me(m.state)??"?",log:w,hot:m.isInHotState===!0||/enters hot state/.test(w)});break;case"SendEvent":{const v=a(me(m.sender)),y=a(me(m.target)),x=i.length;if(i.push({kind:"send",lane:v,to:y,label:me(m.event)??"?",log:w,payload:g}),y!==void 0){const k=s.get(y)??[];k.push({event:me(m.event)??"",key:c(g),step:x}),s.set(y,k)}break}case"DequeueEvent":case"ReceiveEvent":{const v=a(me(m.id)),y=me(m.event)??"?",x=i.length,k={kind:"dequeue",lane:v,label:y,log:w,payload:g};if(v!==void 0){const C=s.get(v)??[];let b=C.findIndex(I=>I.event===y&&I.key===c(g));b<0&&(b=C.findIndex(I=>I.event===y)),b>=0&&(k.peer=C[b].step,i[C[b].step].peer=x,C.splice(b,1))}i.push(k);break}case"MonitorProcessEvent":i.push({kind:"monitor",lane:a(me(m.monitor),!0),to:r.get(me(m.sender)??""),label:me(m.event)??"?",log:w,payload:g});break;case"Announce":i.push({kind:"announce",lane:a(me(m.id)),label:`announce ${me(m.event)}`,log:w,payload:g});break;case"RaiseEvent":case"MonitorRaiseEvent":i.push({kind:"raise",lane:a(me(m.id)??me(m.monitor)),label:`raise ${me(m.event)}`,log:w});break;case"Print":i.push({kind:"print",label:w,log:w});break;case"Halt":i.push({kind:"halt",lane:a(me(m.id)),label:"halt",log:w});break;case"AssertionFailure":{o=me(m.error)??w;const v=[...r.keys()].filter(x=>o.includes(x)).sort((x,k)=>k.length-x.length)[0],y=(d=[...i].reverse().find(x=>x.lane!==void 0))==null?void 0:d.lane;i.push({kind:"error",lane:v!==void 0?r.get(v):y,label:o,log:w});break}case"StrategyDescription":l=me(m.strategyDescription);break;case"GotoState":case"WaitMultipleEvents":break;default:i.push({kind:"note",lane:a(me(m.id)??me(m.monitor)),label:p,log:w,payload:g})}}const u=n.map((p,m)=>m).sort((p,m)=>Number(n[p].monitor)-Number(n[m].monitor)||p-m),h=new Map(u.map((p,m)=>[p,m])),f=p=>p===void 0?void 0:h.get(p);return{lanes:u.map(p=>n[p]),steps:i.map(p=>({...p,lane:f(p.lane),to:f(p.to)})),error:o,strategy:l}}function zP(t){if(t==null)return"";if(typeof t!="object")return String(t);const e=JSON.stringify(t);return e.length>60?e.slice(0,57)+"…":e}const jP=150,VP=96,yn=30,wS=10,br=44,SS=.35,yu=56,kS=2.5,HP=6,uf=t=>e=>e*t+t/2,js=t=>wS+t*yn+yn/2,$g=t=>Math.min(kS,Math.max(SS,t)),Yr=7.2;function Kn(t,e){const n=Math.max(4,Math.floor(e/Yr));return t.length>n?t.slice(0,n-1)+"…":t}function Oi(t,e,n,r,i=7){const s=Math.atan2(r-e,n-t),o=l=>`${n-i*Math.cos(s+l)},${r-i*Math.sin(s+l)}`;return`M${n},${r} L${o(.42)} L${o(-.42)} z`}function UP(t){let e=0;return t.map(n=>n.monitor?"var(--spec)":`var(--lane-${e++%HP})`)}function $P({trace:t,rows:e,cols:n,color:r,focus:i,cursor:s,hover:o,onSelect:l,onHover:a,onToggleLane:c}){const{lanes:u,steps:h}=t,[f]=O.useState(()=>matchMedia("(max-width: 600px)").matches?VP:jP),d=uf(f),p=Math.max(n.length,1)*f,m=wS*2+e.length*yn+yu+16,w=O.useMemo(()=>{const T=new Map(n.map((P,H)=>[P,H])),A=new Map(e.map((P,H)=>[P,H]));return{col:P=>P===void 0?void 0:T.get(P),row:P=>P===void 0?void 0:A.get(P)}},[e,n]),g=O.useRef(null),[v,y]=O.useState(1),[x,k]=O.useState(!1),C=O.useRef(null),b=O.useMemo(()=>{const T=new Map;for(const A of u)T.set(wu(A.id),(T.get(wu(A.id))??0)+1);return new Set([...T].filter(([,A])=>A>1).map(([A])=>A))},[u]),I=O.useMemo(()=>{const T=u.map(A=>A.monitor?0:-1);return e.forEach((A,P)=>{const H=h[A];H.kind==="create"&&H.to!==void 0&&T[H.to]<0&&(T[H.to]=P),H.lane!==void 0&&T[H.lane]<0&&(T[H.lane]=P)}),T.map(A=>Math.max(A,0))},[u,h,e]),W=O.useCallback(()=>{const T=g.current;return T?$g((T.clientWidth-4)/p):1},[p]),N=O.useRef(v),q=O.useCallback((T,A,P)=>{const H=g.current;if(!H)return;const G=N.current,re=$g(T),pe=A??H.clientWidth/2,ke=P??H.clientHeight/2,se=C.current,je=se?se.cx*G-se.sx:H.scrollLeft,ei=se?se.cy*G+br*G-se.sy:H.scrollTop;C.current={cx:(je+pe)/G,cy:(ei+ke-br*G)/G,sx:pe,sy:ke},N.current=re,y(re)},[]);O.useLayoutEffect(()=>{const T=g.current,A=C.current;!T||!A||(C.current=null,T.scrollLeft=A.cx*v-A.sx,T.scrollTop=A.cy*v+br*v-A.sy)},[v]),O.useLayoutEffect(()=>{N.current=1,y(1)},[t,W]),O.useEffect(()=>{const T=g.current;if(!T)return;const A=P=>{if(!P.ctrlKey&&!P.metaKey)return;P.preventDefault();const H=T.getBoundingClientRect(),G=Math.exp(-Math.max(-50,Math.min(50,P.deltaY))*.006);q(N.current*G,P.clientX-H.left,P.clientY-H.top)};return T.addEventListener("wheel",A,{passive:!1}),()=>T.removeEventListener("wheel",A)},[q]);const j=O.useRef(new Map),M=O.useRef(null),$=()=>{const[T,A]=[...j.current.values()];return Math.hypot(T.x-A.x,T.y-A.y)},Q=T=>{T.pointerType==="touch"&&(j.current.set(T.pointerId,{x:T.clientX,y:T.clientY}),j.current.size===2&&(M.current={d:$(),z:N.current}))},de=T=>{if(!j.current.has(T.pointerId))return;j.current.set(T.pointerId,{x:T.clientX,y:T.clientY});const A=M.current,P=g.current;if(!A||j.current.size!==2||!P)return;const[H,G]=[...j.current.values()],re=P.getBoundingClientRect();q(A.z*($()/A.d),(H.x+G.x)/2-re.left,(H.y+G.y)/2-re.top)},ae=T=>{j.current.delete(T.pointerId),j.current.size<2&&(M.current=null)};return O.useEffect(()=>{var ei;const T=g.current;if(!T)return;const A=w.row(s);if(A===void 0)return;const P=br*N.current,H=P+(js(A)-yn)*N.current,G=P+(js(A)+yn)*N.current;H<T.scrollTop+P?T.scrollTop=H-P:G>T.scrollTop+T.clientHeight-yu&&(T.scrollTop=G-T.clientHeight+yu);const re=h[s];if(!re)return;const pe=[re.lane,re.to,re.peer!==void 0?(ei=h[re.peer])==null?void 0:ei.lane:void 0].map(w.col).filter(kr=>kr!==void 0);if(!pe.length)return;const ke=N.current;let se=Math.min(...pe)*f*ke,je=(Math.max(...pe)+1)*f*ke;if(re.kind==="error"){const kr=xS(re.label,uf(f)(pe[0]),p);se=(kr.left-8)*ke,je=(kr.left+kr.w+8)*ke}je-se>T.clientWidth?T.scrollLeft=(se+je)/2-T.clientWidth/2:se<T.scrollLeft?T.scrollLeft=se:je>T.scrollLeft+T.clientWidth&&(T.scrollLeft=je-T.clientWidth)},[s,h,f,p,w]),S.jsxs("div",{className:"seq-wrap",children:[S.jsxs("div",{className:"seq",ref:g,onPointerDown:Q,onPointerMove:de,onPointerUp:ae,onPointerCancel:ae,children:[S.jsx("svg",{className:"seq-head",width:p*v,height:br*v,viewBox:`0 0 ${p} ${br}`,role:"presentation",children:n.map((T,A)=>{const P=u[T];return S.jsxs("g",{className:i.has(T)?"lane focused":"lane",transform:`translate(${d(A)},${br/2})`,style:{"--c":r[T]},onClick:()=>c(T),children:[S.jsx("rect",{x:-f/2+4,y:-15,width:f-8,height:30,rx:15,className:P.monitor?"lane-head monitor":"lane-head"}),S.jsx(KP,{id:P.id,monitor:P.monitor,w:f,showId:b.has(wu(P.id)),focused:i.has(T)})]},P.id)})}),S.jsxs("svg",{width:p*v,height:m*v,viewBox:`0 0 ${p} ${m}`,className:"seq-body",children:[n.map((T,A)=>S.jsx("line",{x1:d(A),x2:d(A),y1:js(I[T])-yn/2,y2:m,className:u[T].monitor?"lifeline monitor":"lifeline",style:{"--c":r[T]}},u[T].id)),e.map(T=>S.jsx(GP,{i:T,s:h[T],W:f,width:p,cursor:s,hover:o===T,color:r,layout:w,onSelect:l,onHover:a},T))]})]}),x&&S.jsx(QP,{onClose:()=>k(!1)}),S.jsxs("div",{className:"zoom-controls",role:"group","aria-label":"Zoom",children:[S.jsx("button",{className:"key-toggle",onClick:()=>k(!x),"aria-expanded":x,children:"Key"}),S.jsx("button",{onClick:()=>q(v/1.25),"aria-label":"Zoom out",disabled:v<=SS,children:"−"}),S.jsxs("button",{className:"zoom-level",onClick:()=>q(v===1?W():1),title:"Toggle 100% / fit width",children:[Math.round(v*100),"%"]}),S.jsx("button",{onClick:()=>q(v*1.25),"aria-label":"Zoom in",disabled:v>=kS,children:"+"})]})]})}function GP({i:t,s:e,W:n,width:r,cursor:i,hover:s,color:o,layout:l,onSelect:a,onHover:c}){const u=uf(n),h=x=>u(l.col(x)??0),f=js(l.row(t)??0),d=t>i,p=t===i,m=e.peer!==void 0&&e.peer===i,w=["row",d&&"future",(p||m)&&"selected",s&&"hover",`k-${e.kind}`].filter(Boolean).join(" "),g=e.lane!==void 0?h(e.lane):0,v=e.lane!==void 0?o[e.lane]:"var(--fg)";let y=null;switch(e.kind){case"create":if(e.lane!==void 0&&e.to!==void 0&&e.lane!==e.to){const x=h(e.to),k=x+(x>g?-6:6);y=S.jsxs(S.Fragment,{children:[S.jsx("line",{x1:g,y1:f,x2:k,y2:f,className:"msg create"}),S.jsx("path",{d:Oi(g,f,k,f),className:"arrowhead create"}),S.jsx("text",{x:(g+x)/2,y:f-5,textAnchor:"middle",className:"msg-label muted",children:"new"})]})}else e.to!==void 0&&(y=S.jsx("circle",{cx:h(e.to),cy:f,r:4,className:"dot",style:{fill:o[e.to]}}));break;case"send":{if(e.lane===void 0||e.to===void 0)break;const x=h(e.to),k=l.row(e.peer),C=k!==void 0?js(k):f,b=x>=g?1:-1;y=e.to===e.lane?S.jsxs(S.Fragment,{children:[S.jsx("path",{d:`M${g},${f} h18 a8,8 0 0 1 8,8 V${C-8} a8,8 0 0 1 -8,8 h-12`,className:"msg",fill:"none"}),S.jsx("path",{d:Oi(g+18,C,g+6,C),className:"arrowhead"}),S.jsx(Gg,{x:g+32,y:f-4,anchor:"start",event:e.label,payload:e.payload,max:130})]}):S.jsxs(S.Fragment,{children:[S.jsx("line",{x1:g,y1:f,x2:x-b*5,y2:C,className:e.peer===void 0?"msg pending":"msg"}),S.jsx("path",{d:Oi(g,f,x-b*5,C),className:"arrowhead"}),S.jsx(Gg,{x:g+b*10,y:f-6,anchor:b>0?"start":"end",event:e.label,payload:e.payload,max:Math.max(Math.abs(x-g)-20,90)})]});break}case"dequeue":y=S.jsx("circle",{cx:g,cy:f,r:4,className:"dot"});break;case"state":{const x=Math.min(n-8,18+e.label.length*Yr),k=Math.min(Math.max(g,x/2+2),r-x/2-2);y=S.jsxs(S.Fragment,{children:[S.jsx("rect",{x:k-x/2,y:f-10,width:x,height:20,rx:10,className:e.hot?"state hot":"state"}),S.jsx("text",{x:k,y:f,dy:"0.35em",textAnchor:"middle",className:"state-label",children:Kn(e.label,n-16)})]});break}case"monitor":{const x=e.to!==void 0?h(e.to):void 0;y=S.jsxs(S.Fragment,{children:[x!==void 0&&S.jsx("line",{x1:x,y1:f,x2:g,y2:f,className:"observe"}),S.jsx("rect",{x:g-5,y:f-5,width:10,height:10,rx:2,transform:`rotate(45 ${g} ${f})`,className:"diamond"}),S.jsx("text",{x:g+11,y:f-6,className:"msg-label",children:Kn(e.label,n-20)})]});break}case"announce":case"raise":case"halt":case"note":y=S.jsxs(S.Fragment,{children:[S.jsx("rect",{x:g-4,y:f-4,width:8,height:8,rx:2,className:"dot"}),S.jsx("text",{x:g+9,y:f,dy:"0.35em",className:"note-label",children:Kn(e.label,n-20)})]});break;case"print":y=S.jsxs(S.Fragment,{children:[S.jsx("rect",{x:6,y:f-9,width:Math.min(r-12,8+Kn(`print: ${e.label}`,r-20).length*Yr),height:18,rx:4,className:"label-bg"}),S.jsx("text",{x:10,y:f,dy:"0.35em",className:"print-label",children:Kn(`print: ${e.label}`,r-20)})]});break;case"error":{const{left:x,w:k}=xS(e.label,e.lane!==void 0?g:void 0,r);y=S.jsxs(S.Fragment,{children:[S.jsx("rect",{x,y:f-12,width:k,height:24,rx:12,className:"error-bar"}),S.jsx("text",{x:x+12,y:f,dy:"0.35em",className:"error-label",children:CS(e.label)})]});break}}return S.jsxs("g",{className:w,"data-row":t,onClick:()=>a(t),onMouseEnter:()=>c(t),onMouseLeave:()=>c(void 0),style:{"--c":v},children:[S.jsx("rect",{x:0,y:f-yn/2,width:r,height:yn,className:"row-hit"}),p&&S.jsx("rect",{x:0,y:f-yn/2+3,width:3,height:yn-6,rx:1.5,className:"row-cursor"}),S.jsx("title",{children:e.log}),y]})}function xS(t,e,n){const r=Math.min(n-8,24+CS(t).length*7.8);return{left:Math.max(4,Math.min(e!==void 0?e-16:4,n-r-4)),w:r}}function CS(t){const e=/^([^:.]+)/.exec(t);return`✕ ${((e==null?void 0:e[1])??t).trim()}`}const wu=t=>t.replace(/\(\d+\)$/,"");function KP({id:t,monitor:e,w:n,showId:r,focused:i}){const s=/^(.*?)(\(\d+\))?$/.exec(t),o=r?s[2]??"":"",l=n-14,a=(s[1].length+o.length)*Yr,c=Math.max(.82,Math.min(1,l/a));return S.jsxs("text",{textAnchor:"middle",dy:"0.35em",className:"lane-label",style:c<1?{fontSize:`${(12*c).toFixed(1)}px`}:void 0,children:[S.jsx("title",{children:`${e?`spec ${t}`:t} (${i?"click to stop focusing":"click to show only its steps"})`}),Kn(s[1],l/c-o.length*Yr),o&&S.jsx("tspan",{className:"lane-id",children:o})]})}function Gg({x:t,y:e,anchor:n,event:r,payload:i,max:s}){const o=Kn(r,s),l=s-o.length*Yr-6,a=i!==void 0&&l>30?Kn(zP(i),l):"",c=(o.length+(a?a.length+1:0))*Yr+8,u=n==="middle"?t-c/2:n==="end"?t-c+4:t-4;return S.jsxs(S.Fragment,{children:[S.jsx("rect",{x:u,y:e-11,width:c,height:15,rx:4,className:"label-bg"}),S.jsxs("text",{x:t,y:e,textAnchor:n,className:"msg-label",children:[S.jsx("tspan",{className:"ev",children:o}),a&&S.jsxs("tspan",{className:"payload",children:[" ",a]})]})]})}function QP({onClose:t}){const e=(n,r)=>S.jsxs("li",{children:[S.jsx("svg",{width:"40",height:"18",viewBox:"0 0 40 18","aria-hidden":"true",children:n}),S.jsx("span",{children:r})]});return S.jsxs("div",{className:"legend",role:"dialog","aria-label":"Diagram key",children:[S.jsxs("div",{className:"legend-head",children:[S.jsx("strong",{children:"Key"}),S.jsx("button",{className:"ghost",onClick:t,"aria-label":"Close key",children:"✕"})]}),S.jsxs("ul",{children:[e(S.jsxs(S.Fragment,{children:[S.jsx("line",{x1:"4",y1:"9",x2:"30",y2:"9",className:"msg",style:{"--c":"var(--lane-0)"}}),S.jsx("path",{d:Oi(4,9,34,9),className:"arrowhead",style:{"--c":"var(--lane-0)"}})]}),"Message, in the sender's color; it ends where the receiver handles it"),e(S.jsxs(S.Fragment,{children:[S.jsx("line",{x1:"4",y1:"9",x2:"30",y2:"9",className:"msg pending",style:{"--c":"var(--lane-0)"}}),S.jsx("path",{d:Oi(4,9,34,9),className:"arrowhead",style:{"--c":"var(--lane-0)"}})]}),"Sent but never handled"),e(S.jsxs(S.Fragment,{children:[S.jsx("line",{x1:"4",y1:"9",x2:"30",y2:"9",className:"msg create"}),S.jsx("path",{d:Oi(4,9,34,9),className:"arrowhead create"})]}),"Created a machine"),e(S.jsx("rect",{x:"4",y:"2",width:"32",height:"14",rx:"7",className:"state",style:{"--c":"var(--lane-1)"}}),"Entered a state"),e(S.jsx("rect",{x:"4",y:"2",width:"32",height:"14",rx:"7",className:"state hot"}),"Spec in a hot state (must leave it eventually)"),e(S.jsxs(S.Fragment,{children:[S.jsx("line",{x1:"4",y1:"9",x2:"26",y2:"9",className:"observe"}),S.jsx("rect",{x:"26",y:"4",width:"10",height:"10",rx:"2",transform:"rotate(45 31 9)",className:"diamond"})]}),"Spec observed an event"),e(S.jsx("rect",{x:"4",y:"3",width:"32",height:"12",rx:"6",className:"error-bar"}),"Where the bug was detected"),e(S.jsx("rect",{x:"4",y:"2",width:"32",height:"14",rx:"7",className:"lane-head",style:{"--c":"var(--lane-0)"}}),"Tap a machine's name to show only its steps")]})]})}const YP=/([\w./-]+\.p):(\d+):(\d+)/,XP=820,JP=560;function ZP({json:t,view:e,onOpenLocation:n,onShowDiagram:r}){const i=O.useMemo(()=>FP(t),[t]),s=O.useMemo(()=>UP(i.lanes),[i]),[o,l]=O.useState(i.steps.length-1),[a,c]=O.useState(!1),[u,h]=O.useState(!1),[f,d]=O.useState(new Set),[p,m]=O.useState(),w=O.useRef(null),[g,v]=O.useState("none"),{rows:y,cols:x}=O.useMemo(()=>WP(i,f),[i,f]),k=y.indexOf(o),C=y.length-1;O.useEffect(()=>{l(i.steps.length-1),c(!1),d(new Set)},[i]),O.useEffect(()=>{if(k>=0||!y.length)return;const M=y.filter($=>$<o);l(M.length?M[M.length-1]:y[0])},[k,y,o]),O.useEffect(()=>{if(!a)return;if(k>=C){c(!1);return}const M=setTimeout(()=>l(y[Math.min(k+1,C)]),220);return()=>clearTimeout(M)},[a,k,C,y]),O.useLayoutEffect(()=>{const M=w.current;if(!M)return;const $=new ResizeObserver(()=>v(M.clientWidth>=XP?"side":M.clientHeight>=JP&&!matchMedia("(max-width: 900px)").matches?"bottom":"none"));return $.observe(M),()=>$.disconnect()},[]);const b=M=>{M!==void 0&&(c(!1),l(M))},I=M=>b(y[Math.max(0,Math.min(M,C))]),W=M=>d($=>{const Q=new Set($);return Q.has(M)?Q.delete(M):Q.add(M),Q}),N=i.steps[o],q=i.error?YP.exec(i.error):null,j=S.jsx(eE,{trace:i,rows:y,color:s,cursor:o,hover:p,onSelect:M=>{b(M),e==="log"&&r()},onHover:m});return S.jsxs("div",{className:"trace",ref:w,children:[i.error&&S.jsxs("div",{className:u?"trace-error open":"trace-error",onClick:()=>h(!u),children:[S.jsxs("span",{className:"trace-error-text",children:[S.jsx("strong",{children:"Bug:"})," ",i.error]}),q&&S.jsxs("button",{className:"link",onClick:M=>{M.stopPropagation(),n(q[1],Number(q[2]))},children:[S.jsxs("span",{className:"goto-long",children:["Go to ",q[1],":",q[2]]}),S.jsxs("span",{className:"goto-short",children:["line ",q[2]," →"]})]})]}),e==="diagram"&&S.jsxs("div",{className:"trace-controls",children:[S.jsx("button",{onClick:()=>I(0),title:"First step","aria-label":"First step",className:"jump",children:"⏮"}),S.jsx("button",{onClick:()=>I(k-1),title:"Previous step","aria-label":"Previous step",children:"◀"}),S.jsx("button",{className:"primary",onClick:()=>{!a&&k>=C&&l(y[0]),c(!a)},title:a?"Pause":"Play",children:a?"Pause":"Play"}),S.jsx("button",{onClick:()=>I(k+1),title:"Next step","aria-label":"Next step",children:"▶"}),S.jsx("button",{onClick:()=>I(C),title:"Last step","aria-label":"Last step",className:"jump",children:"⏭"}),S.jsx("input",{type:"range",min:0,max:Math.max(C,0),value:Math.max(k,0),onChange:M=>I(Number(M.target.value)),"aria-label":"Step"}),S.jsxs("span",{className:"muted",children:[k+1,"/",y.length]})]}),f.size>0&&S.jsxs("div",{className:"trace-focus",children:[S.jsx("span",{className:"muted",children:"Only steps of"}),[...f].map(M=>S.jsxs("button",{className:"chip",style:{"--c":s[M]},onClick:()=>W(M),"aria-label":`Stop focusing ${i.lanes[M].id}`,children:[S.jsx("span",{className:"chip-name",children:i.lanes[M].id})," ",S.jsx("span",{"aria-hidden":"true",children:"✕"})]},M)),S.jsx("button",{className:"link",onClick:()=>d(new Set),children:"Show all"})]}),e==="log"?j:S.jsxs(S.Fragment,{children:[S.jsxs("div",{className:`trace-main list-${g}`,children:[S.jsx($P,{trace:i,rows:y,cols:x,color:s,focus:f,cursor:o,hover:p,onSelect:b,onHover:m,onToggleLane:W}),g!=="none"&&j]}),N&&g==="none"&&S.jsxs("div",{className:"step-detail",children:[S.jsx("div",{children:N.log}),N.payload!==void 0&&S.jsx("pre",{children:JSON.stringify(N.payload,null,2)})]})]})]})}function eE({trace:t,rows:e,color:n,cursor:r,hover:i,onSelect:s,onHover:o}){const l=O.useRef(null);return O.useEffect(()=>{const a=l.current,c=a==null?void 0:a.querySelector(`[data-step="${r}"]`);if(!a||!c)return;const u=c.offsetTop-a.offsetTop;u<a.scrollTop?a.scrollTop=u-8:u+c.offsetHeight>a.scrollTop+a.clientHeight&&(a.scrollTop=u+c.offsetHeight-a.clientHeight+8)},[r,e]),S.jsx("ol",{className:"step-list",ref:l,"aria-label":"Steps",children:e.map(a=>{const c=t.steps[a],u=[a===r&&"current",a===i&&"hover",c.kind==="error"&&"error"].filter(Boolean).join(" ");return S.jsx("li",{"data-step":a,className:u,children:S.jsxs("button",{onClick:()=>s(a),onMouseEnter:()=>o(a),onMouseLeave:()=>o(void 0),"aria-current":a===r?"step":void 0,children:[S.jsx("span",{className:"n",children:a+1}),S.jsx("span",{className:"swatch",style:{background:c.lane!==void 0?n[c.lane]:"var(--muted)"}}),S.jsx("span",{className:"text",children:c.log})]})},a)})})}const Su=[{label:"random",args:[]},{label:"pct (10)",args:["--sch-pct","10"]},{label:"pos",args:["--sch-pos"]},{label:"feedback pct (10)",args:["--sch-feedbackpct","10"]}],Kg=()=>Math.floor(Math.random()*1e6),tE=t=>t.split(/\s+/).filter(Boolean),nE="https://github.com/crowecawcaw/p-rs",ys="(max-width: 900px)",rE={files:"Files",console:"Console",trace:"Trace",log:"Log"};function iE(){const t=O.useMemo(()=>new qP,[]),[e,n]=O.useState(xr[0]),[r,i]=O.useState({}),[s,o]=O.useState(""),[l,a]=O.useState([]),[c,u]=O.useState(xr[0].testCase),[h,f]=O.useState(xr[0].schedules),[d,p]=O.useState(0),[m,w]=O.useState(Kg),[g,v]=O.useState(!1),[y,x]=O.useState(""),[k,C]=O.useState([]),[b,I]=O.useState(""),[W,N]=O.useState(),[q,j]=O.useState(!1),[M,$]=O.useState(),[Q,de]=O.useState(),[ae,T]=O.useState(()=>matchMedia(ys).matches),[A,P]=O.useState(()=>matchMedia(ys).matches?"files":"console"),[H,G]=O.useState(),[re,pe]=O.useState(!1),ke=O.useRef(!1),[se,je]=O.useState(!1),[ei,kr]=O.useState(!1),Ud=O.useRef(null),[rs,$d]=O.useState(!1),gc=O.useCallback(L=>{const Y=pb(L);n(L),i(Y),o(Y[L.open]!==void 0?L.open:lm(Object.keys(Y))[0]),u(L.testCase),f(L.schedules),x(L.extraArgs??""),p(0),$(void 0),de(void 0),P(matchMedia(ys).matches?"files":"console"),ke.current=!1},[]);O.useEffect(()=>{const L=Ud.current;if(!L)return;const Y=()=>kr(L.scrollHeight>L.clientHeight+1);Y();const ie=new ResizeObserver(Y);return ie.observe(L),()=>ie.disconnect()},[e]),O.useEffect(()=>{const L=matchMedia(ys),Y=()=>{T(L.matches),L.matches||P(ie=>ie==="files"?"console":ie)};return L.addEventListener("change",Y),()=>L.removeEventListener("change",Y)},[]),O.useEffect(()=>{const L=new URLSearchParams(location.search).get("demo");gc(xr.find(Y=>Y.id===L)??xr[0]),pe(!0)},[gc]),O.useEffect(()=>{if(!re||Object.keys(r).length===0)return;let L=!1;const Y=setTimeout(async()=>{try{const ie=await t.compile(r);if(L)return;C(ie.diagnostics),I(ie.console),N(ie.ok),ie.ok&&(a(ie.tests),u(is=>ie.tests.includes(is)?is:ie.tests[0]??""))}catch{}},400);return()=>{L=!0,clearTimeout(Y)}},[r,re,t]);const Gd=O.useCallback(async()=>{if(q||!c)return;const L=g?m:Kg();w(L),j(!0),de(void 0);try{const Y=await t.check({files:r,testCase:c,schedules:h,seed:L,extraArgs:[...Su[d].args,...tE(y)]});$(Y),Y.diagnostics.length&&C(Y.diagnostics),P(Y.bug&&Y.traceJson?"trace":"console")}catch(Y){const ie=String(Y.message??Y);de(ie==="cancelled"?"Stopped.":`The checker crashed: ${ie}`),$(void 0),P("console")}finally{j(!1)}},[q,c,g,m,t,r,h,d,y]),vc=O.useCallback((L,Y)=>{const ie=L.replace(/^\/work\//,""),is=r[ie]!==void 0?ie:Object.keys(r).find(Yd=>Yd.endsWith("/"+ie)||Yd===ie);is&&(o(is),G({line:Y,nonce:Date.now()}),matchMedia(ys).matches&&P("files"))},[r]),bS=L=>{if(ke.current&&!confirm("Discard your edits and load another demo?"))return;const Y=xr.find(ie=>ie.id===L);history.replaceState(null,"",`?demo=${Y.id}`),gc(Y)},TS=()=>{var ie;const L=(ie=prompt("New file name","PSrc/NewMachine.p"))==null?void 0:ie.trim();if(!L)return;const Y=L.endsWith(".p")||L.includes(".")?L:`${L}.p`;r[Y]===void 0&&i({...r,[Y]:""}),o(Y),ke.current=!0},RS=O.useMemo(()=>lm(Object.keys(r)),[r]),MS=O.useMemo(()=>{const L=new Map;for(const Y of k)L.set(Y.file,(L.get(Y.file)??0)+1);return L},[k]),IS=q?"Checking…":Q||(W===!1||M&&!M.ok?"Compile error":M?M.bug?"Bug found":`No bugs in ${h} schedules`:W?"Ready":"Loading…"),AS=q?"":W===!1||M!=null&&M.bug||M&&!M.ok?"bad":M!=null&&M.ok?"good":"",Kd=S.jsxs("div",{className:se?"blurb open":"blurb",children:[S.jsx("p",{ref:Ud,children:e.blurb}),(ei||se)&&S.jsx("button",{className:"more",onClick:()=>je(!se),children:se?"less":"more"})]}),Qd=!q&&(W===!1||M!==void 0&&!M.ok)&&k.length>0,PS=ae?q||!!Q||Qd:q||!!M||!!Q||W===!1,ES=(M==null?void 0:M.schedule)&&!q&&S.jsxs("span",{className:"downloads",children:[S.jsx(ku,{name:`${M.testCase}.schedule`,text:M.schedule}),M.traceText&&S.jsx(ku,{name:`${M.testCase}.txt`,text:M.traceText}),M.traceJson&&S.jsx(ku,{name:`${M.testCase}.trace.json`,text:M.traceJson})]});return S.jsxs("div",{className:"app",children:[S.jsxs("header",{className:"top",children:[S.jsxs("div",{className:"brand",children:[S.jsx("span",{className:"logo",children:"p-rs"}),S.jsx("span",{className:"brand-name",children:"Playground"})]}),S.jsx("select",{className:"demo-select",value:e.id,onChange:L=>bS(L.target.value),"aria-label":"Demo",children:xr.map(L=>S.jsx("option",{value:L.id,children:L.title},L.id))}),S.jsx("div",{className:"spacer"}),S.jsxs("a",{className:"button ghost gh",href:nE,target:"_blank",rel:"noreferrer","aria-label":"GitHub",children:[S.jsx("svg",{width:"20",height:"20",viewBox:"0 0 16 16","aria-hidden":"true",fill:"currentColor",children:S.jsx("path",{d:"M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"})}),S.jsx("span",{className:"gh-text",children:"GitHub"})]})]}),!ae&&Kd,S.jsxs("main",{className:`panes tab-${A}`,children:[S.jsxs("section",{className:"pane editor-pane",children:[ae&&Kd,S.jsxs("div",{className:"file-tabs",role:"tablist",children:[RS.map(L=>{const Y=L.lastIndexOf("/"),ie=MS.get(L);return S.jsxs("button",{role:"tab","aria-selected":L===s,className:L===s?"file-tab active":"file-tab",onClick:()=>o(L),title:L,children:[Y>=0&&S.jsx("span",{className:"dir",children:L.slice(0,Y+1)}),L.slice(Y+1),ie?S.jsx("span",{className:"badge",children:ie}):null]},L)}),S.jsx("button",{className:"file-tab add",onClick:TS,title:"New file","aria-label":"New file",children:"+"})]}),s&&r[s]!==void 0&&S.jsx(_P,{path:s,value:r[s],diagnostics:k.filter(L=>L.file===s),jump:H,onChange:(L,Y)=>{ke.current=!0,i(ie=>({...ie,[L]:Y}))},onRun:Gd})]}),S.jsxs("section",{className:"pane run-pane",children:[S.jsxs("div",{className:rs?"controls options-open":"controls",children:[S.jsxs("div",{className:"run-bar",children:[S.jsxs("label",{className:"tc",children:[S.jsx("span",{className:"lbl",children:"Test case"}),S.jsxs("select",{value:c,onChange:L=>u(L.target.value),disabled:!l.length,"aria-label":"Test case",children:[l.length===0&&S.jsx("option",{value:c,children:c||"—"}),l.map(L=>S.jsx("option",{value:L,children:L},L))]})]}),q?S.jsx("button",{className:"danger run",onClick:()=>t.cancel(),children:"Stop"}):S.jsx("button",{className:"primary run",onClick:Gd,disabled:!c||W===!1,title:"Ctrl/⌘+Enter",children:W!==!1?"Check":k.length===1?"Fix 1 error":k.length?`Fix ${k.length} errors`:"Fix errors"})]}),S.jsxs("details",{className:"options",open:rs,onToggle:L=>$d(L.target.open),children:[S.jsxs("summary",{children:[S.jsx("svg",{className:"chevron",width:"10",height:"10",viewBox:"0 0 10 10","aria-hidden":"true",children:S.jsx("path",{d:"M3 1.5 L6.5 5 L3 8.5",fill:"none",stroke:"currentColor",strokeWidth:"1.6",strokeLinecap:"round",strokeLinejoin:"round"})}),"Options",S.jsxs("span",{className:"muted",children:[" ","· ",h," schedules · ",Su[d].label,g?` · seed ${m}`:"",y.trim()?` · ${y.trim()}`:""]})]}),S.jsxs("div",{className:"options-grid",children:[S.jsxs("label",{children:["Schedules",S.jsx("input",{type:"number",inputMode:"numeric",min:1,max:1e6,value:h,onChange:L=>f(Math.max(1,Number(L.target.value)||1))})]}),S.jsxs("label",{children:["Strategy",S.jsx("select",{value:d,onChange:L=>p(Number(L.target.value)),children:Su.map((L,Y)=>S.jsx("option",{value:Y,children:L.label},L.label))})]}),S.jsxs("label",{children:["Seed",S.jsx("input",{type:"number",inputMode:"numeric",value:m,onChange:L=>{w(Number(L.target.value)||0),v(!0)}})]}),S.jsxs("label",{className:"check",children:[S.jsx("input",{type:"checkbox",checked:g,onChange:L=>v(L.target.checked)}),"Keep seed"]}),S.jsxs("label",{className:"span-all",children:["Extra flags",S.jsx("input",{value:y,placeholder:"e.g. --timed --max-steps 500",onChange:L=>x(L.target.value),spellCheck:!1,autoCapitalize:"off",autoCorrect:"off"})]})]})]})]}),S.jsxs("div",{className:"out-head",children:[S.jsxs("div",{className:"out-tabs",role:"tablist",children:[(ae?["files","console","trace","log"]:["console","trace","log"]).map(L=>S.jsxs("button",{role:"tab","aria-selected":A===L,className:A===L?"out-tab active":"out-tab",onClick:()=>P(L),disabled:(L==="trace"||L==="log")&&!(M!=null&&M.traceJson),children:[rE[L],L==="files"&&k.length>0&&S.jsx("span",{className:"badge",children:k.length}),L==="trace"&&(M==null?void 0:M.bug)&&S.jsx("span",{className:"bug-dot","aria-label":"bug found"})]},L)),ae&&S.jsx("button",{className:rs?"out-tab gear active":"out-tab gear",onClick:()=>$d(!rs),"aria-expanded":rs,"aria-label":"Checker options",title:"Checker options",children:S.jsxs("svg",{width:"20",height:"20",viewBox:"0 0 24 24","aria-hidden":"true",fill:"none",stroke:"currentColor",strokeWidth:"1.8",strokeLinecap:"round",strokeLinejoin:"round",children:[S.jsx("path",{d:"M4 6h10M18 6h2M4 12h4M12 12h8M4 18h12M20 18h0"}),S.jsx("circle",{cx:"16",cy:"6",r:"2"}),S.jsx("circle",{cx:"10",cy:"12",r:"2"}),S.jsx("circle",{cx:"18",cy:"18",r:"2"})]})})]}),PS&&S.jsxs("div",{className:"status-line",children:[S.jsx("span",{className:`status ${AS}`,children:IS}),!q&&(W===!1||M&&!M.ok)&&k[0]&&S.jsxs("button",{className:"diag",onClick:()=>vc(k[0].file,k[0].line),children:[k[0].file,":",k[0].line," ",k[0].message]}),M&&!q&&!Qd&&S.jsxs("span",{className:"muted",children:[(M.millis/1e3).toFixed(2)," s"]}),!ae&&ES]})]}),S.jsxs("div",{className:"output",children:[A==="console"&&S.jsx(lE,{summary:!Q&&(M!=null&&M.ok)?M.bug?`✕ ${oE(M)}`:`✓ No bugs found in ${M.testCase}`:void 0,bad:M==null?void 0:M.bug,text:Q??(M?M.console:W===!1?b:""),onOpenLocation:vc,empty:W===!1?"":"Press Check (Ctrl/⌘+Enter) to explore schedules of the selected test case."}),(A==="trace"||A==="log")&&(M==null?void 0:M.traceJson)&&S.jsx(ZP,{json:M.traceJson,view:A==="log"?"log":"diagram",onOpenLocation:vc,onShowDiagram:()=>P("trace")})]})]})]})]})}const sE=/([\w./-]+\.p):(\d+)(?::(\d+))?/g;function oE(t){const e=`${t.traceText??""}
${t.console}`.split(`
`).find(n=>/<ErrorLog>|Assertion Failed|Deadlock|liveness|Exception/i.test(n));return e?e.replace(/^.*?<ErrorLog>\s*/,"").trim():"Bug found"}function lE({text:t,empty:e,summary:n,bad:r,onOpenLocation:i}){return t?S.jsxs("pre",{className:"console",children:[n&&S.jsx("div",{className:r?"summary bad":"summary good",children:n}),t.split(`
`).map((s,o)=>{const l=/Error|Assertion Failed|\bbug\b(?! found: 0)|Found [1-9]/i.test(s)&&!/Found 0 bug/.test(s)?"line bad":/succeeded|Found 0 bug/.test(s)?"line good":s.startsWith("$ ")?"line cmd":"line",a=[];let c=0;for(const u of s.matchAll(sE)){a.push(s.slice(c,u.index));const[h,f,d]=u;a.push(S.jsx("button",{className:"loc",onClick:()=>i(f,Number(d)),children:h},u.index)),c=(u.index??0)+h.length}return a.push(s.slice(c)),S.jsx("div",{className:l,children:a},o)})]}):S.jsx("div",{className:"console empty",children:e})}function ku({name:t,text:e}){const n=O.useMemo(()=>URL.createObjectURL(new Blob([e],{type:"text/plain"})),[e]);return O.useEffect(()=>()=>URL.revokeObjectURL(n),[n]),S.jsxs("a",{className:"download",href:n,download:t,title:`Download ${t}`,children:["⤓ ",t.slice(t.indexOf(".")+1)]})}ly(document.getElementById("root")).render(S.jsx(O.StrictMode,{children:S.jsx(iE,{})}));
