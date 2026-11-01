/**
 * The phone's bridge.
 *
 * The web app is the same app on both hosts: the desktop rig talks to the
 * router over loopback HTTP, and the phone talks to the Rust core in the
 * same process. The app does not know which — it calls one function, and
 * the transport behind it differs. This file is the phone's half of that
 * seam.
 *
 * **One function.**
 *
 *     window.__stride_bridge__.apiCall({ method, path, body })
 *
 * It answers with the response body as a JSON string, and it rejects when
 * the request failed. That is the whole contract, and it is the shape the
 * web app's transport expects.
 *
 * **It is installed before the app boots.** Capacitor injects this file
 * into the page it serves, ahead of the app's own scripts, so the first
 * request the app makes already finds the bridge. The native shell asks for
 * the injection (`Bridge.injectScriptBeforeLoad`), and this file arrives
 * with Capacitor's own runtime — after it, so the plugin proxy it calls
 * already exists.
 *
 * **It adds no capability.** The route table, the refusal of the Garmin-key
 * routes, the lock check and the redaction all live in the native plugin
 * this file calls. The file moves the method into the body, hands the
 * request over and hands the answer back. It cannot reach anything the
 * plugin would refuse, because the plugin is what refuses.
 *
 * **No JSON, no Promise, no modern syntax.** The file is ES5 on purpose: it
 * is injected into a page by a native runtime, and the unit test that
 * proves its shape runs it on a JavaScript engine that is not a browser.
 */
(function (global) {
  'use strict';

  var PLUGIN_ID = 'StrideApi';

  /** The native plugin, or null when it is not there. */
  function nativePlugin() {
    var capacitor = global.Capacitor;
    if (!capacitor || !capacitor.Plugins) {
      return null;
    }
    return capacitor.Plugins[PLUGIN_ID] || null;
  }

  /**
   * The body of the request, with the method in it.
   *
   * The plugin's contract is `apiCall(path, body)`: the method travels
   * inside the body, because a body is what a request carries and a method
   * is how it is made. So the method is put in — the app's shape adapts to
   * the plugin's, not the other way round.
   *
   * The app's body is copied rather than used: the app's own object is
   * not ours to change.
   */
  function bodyWithMethod(body, method) {
    var copy = {};
    if (body !== null && body !== undefined && typeof body === 'object') {
      for (var key in body) {
        if (Object.prototype.hasOwnProperty.call(body, key)) {
          copy[key] = body[key];
        }
      }
    } else if (body !== null && body !== undefined) {
      // A body that is not an object is carried under "body", so nothing is
      // lost and the shape stays one shape.
      copy.body = body;
    }
    copy.method = method || 'GET';
    return copy;
  }

  /**
   * A promise that has already failed.
   *
   * The engine's own `Promise` is not used: the file is injected into a
   * page by a native runtime, and the test that proves it runs it on an
   * engine that is not a browser. A thenable is enough — `await` accepts
   * one — and the failure still reaches the app as a failure.
   */
  function failed(reason) {
    return {
      then: function (onFulfilled, onRejected) {
        if (typeof onRejected === 'function') {
          return onRejected(reason);
        }
        return this;
      }
    };
  }

  global.__stride_bridge__ = {
    /**
     * Call a route in the Rust core.
     *
     * `request` is `{ method, path, body }` — the shape the web app uses.
     * The answer is the response body as a JSON string.
     */
    apiCall: function (request) {
      var args = request || {};
      var native = nativePlugin();

      if (!native || typeof native.apiCall !== 'function') {
        return failed(new Error('the native bridge is not installed'));
      }

      return native
        .apiCall({
          path: args.path,
          body: bodyWithMethod(args.body, args.method)
        })
        .then(function (answer) {
          if (answer && typeof answer.json === 'string') {
            return answer.json;
          }
          // The plugin answers with `{ json: "…" }`. Anything else is a bug
          // in the shell, and a bug must not look like an answer.
          return failed(new Error('the native bridge answered with no JSON'));
        });
    }
  };
})(typeof window !== 'undefined' ? window : this);
