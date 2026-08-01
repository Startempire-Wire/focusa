typeof window < "u" && ((window.__svelte ??= {}).v ??= /* @__PURE__ */ new Set()).add("5");
const Jo = 2, Hi = "[", Yi = "[!", Bs = "[?", Gi = "]", Gt = {}, ne = Symbol("uninitialized"), Xo = "http://www.w3.org/1999/xhtml", Ji = !1;
var Xi = Array.isArray, Qo = Array.prototype.indexOf, Hr = Array.prototype.includes, Ko = Array.from, Yr = Object.keys, Gr = Object.defineProperty, Bt = Object.getOwnPropertyDescriptor, el = Object.getOwnPropertyDescriptors, tl = Object.prototype, rl = Array.prototype, Qi = Object.getPrototypeOf, qs = Object.isExtensible;
const nl = () => {
};
function sl(t) {
  for (var e = 0; e < t.length; e++)
    t[e]();
}
function Ki() {
  var t, e, r = new Promise((n, s) => {
    t = n, e = s;
  });
  return { promise: r, resolve: t, reject: e };
}
const oe = 2, Jt = 4, gn = 8, ea = 1 << 24, De = 16, Xe = 32, Qe = 64, Gn = 128, _e = 512, se = 1024, ie = 2048, Ze = 4096, Ae = 8192, we = 16384, Pt = 32768, Hs = 1 << 25, Xt = 65536, Jr = 1 << 17, il = 1 << 18, Dt = 1 << 19, al = 1 << 20, $t = 65536, Xr = 1 << 21, qt = 1 << 22, ot = 1 << 23, jn = Symbol("$state"), ol = Symbol("legacy props"), ll = Symbol(""), ta = Symbol("attributes"), Jn = Symbol("class"), Xn = Symbol("style"), Qn = Symbol("text"), vn = new class extends Error {
  name = "StaleReactionError";
  message = "The reaction that called `getAbortSignal()` was re-run or destroyed";
}(), cl = (
  // We gotta write it like this because after downleveling the pure comment may end up in the wrong location
  !!globalThis.document?.contentType && /* @__PURE__ */ globalThis.document.contentType.includes("xml")
), ms = 3, bn = 8;
function ul(t) {
  throw new Error("https://svelte.dev/e/lifecycle_outside_component");
}
function dl() {
  throw new Error("https://svelte.dev/e/async_derived_orphan");
}
function fl() {
  throw new Error("https://svelte.dev/e/effect_update_depth_exceeded");
}
function hl() {
  throw new Error("https://svelte.dev/e/hydration_failed");
}
function pl() {
  throw new Error("https://svelte.dev/e/state_descriptors_fixed");
}
function ml() {
  throw new Error("https://svelte.dev/e/state_prototype_fixed");
}
function gl() {
  throw new Error("https://svelte.dev/e/state_unsafe_mutation");
}
function vl() {
  throw new Error("https://svelte.dev/e/svelte_boundary_reset_onerror");
}
function bl() {
  console.warn("https://svelte.dev/e/derived_inert");
}
function yn(t) {
  console.warn("https://svelte.dev/e/hydration_mismatch");
}
function yl() {
  console.warn("https://svelte.dev/e/svelte_boundary_reset_noop");
}
let Q = !1;
function Ut(t) {
  Q = t;
}
let ee;
function Ue(t) {
  if (t === null)
    throw yn(), Gt;
  return ee = t;
}
function gs() {
  return Ue(/* @__PURE__ */ mt(ee));
}
function Te(t) {
  if (Q) {
    if (/* @__PURE__ */ mt(ee) !== null)
      throw yn(), Gt;
    ee = t;
  }
}
function _l(t = 1) {
  if (Q) {
    for (var e = t, r = ee; e--; )
      r = /** @type {TemplateNode} */
      /* @__PURE__ */ mt(r);
    ee = r;
  }
}
function ra(t = !0) {
  for (var e = 0, r = ee; ; ) {
    if (r.nodeType === bn) {
      var n = (
        /** @type {Comment} */
        r.data
      );
      if (n === Gi) {
        if (e === 0) return r;
        e -= 1;
      } else (n === Hi || n === Yi || // "[1", "[2", etc. for if blocks
      n[0] === "[" && !isNaN(Number(n.slice(1)))) && (e += 1);
    }
    var s = (
      /** @type {TemplateNode} */
      /* @__PURE__ */ mt(r)
    );
    t && r.remove(), r = s;
  }
}
function Al(t) {
  if (!t || t.nodeType !== bn)
    throw yn(), Gt;
  return (
    /** @type {Comment} */
    t.data
  );
}
function na(t) {
  return t === this.v;
}
function wl(t, e) {
  return t != t ? e == e : t !== e || t !== null && typeof t == "object" || typeof t == "function";
}
function kl(t) {
  return !wl(t, this.v);
}
let $l = !1, ke = null;
function Qt(t) {
  ke = t;
}
function R(t, e = !1, r) {
  ke = {
    p: ke,
    i: !1,
    c: null,
    e: null,
    s: t,
    x: null,
    r: (
      /** @type {Effect} */
      D
    ),
    l: null
  };
}
function L(t) {
  var e = (
    /** @type {ComponentContext} */
    ke
  ), r = e.e;
  if (r !== null) {
    e.e = null;
    for (var n of r)
      Bl(n);
  }
  return t !== void 0 && (e.x = t), e.i = !0, ke = e.p, t ?? /** @type {T} */
  {};
}
function sa() {
  return !0;
}
let yt = [];
function ia() {
  var t = yt;
  yt = [], sl(t);
}
function Ht(t) {
  if (yt.length === 0 && !kr) {
    var e = yt;
    queueMicrotask(() => {
      e === yt && ia();
    });
  }
  yt.push(t);
}
function Sl() {
  for (; yt.length > 0; )
    ia();
}
function aa(t) {
  var e = D;
  if (e === null)
    return j.f |= ot, t;
  if ((e.f & Pt) === 0 && (e.f & Jt) === 0)
    throw t;
  at(t, e);
}
function at(t, e) {
  if (!(e !== null && (e.f & we) !== 0)) {
    for (; e !== null; ) {
      if ((e.f & Gn) !== 0) {
        if ((e.f & Pt) === 0)
          throw t;
        try {
          e.b.error(t);
          return;
        } catch (r) {
          t = r;
        }
      }
      e = e.parent;
    }
    throw t;
  }
}
const xl = -7169;
function te(t, e) {
  t.f = t.f & xl | e;
}
function vs(t) {
  (t.f & _e) !== 0 || t.deps === null ? te(t, se) : te(t, Ze);
}
function oa(t) {
  if (t !== null)
    for (const e of t)
      (e.f & oe) === 0 || (e.f & $t) === 0 || (e.f ^= $t, oa(
        /** @type {Derived} */
        e.deps
      ));
}
function la(t, e, r) {
  (t.f & ie) !== 0 ? e.add(t) : (t.f & Ze) !== 0 && r.add(t), oa(t.deps), te(t, se);
}
function Cl(t) {
  let e = 0, r = Lr(0), n;
  return () => {
    ks() && (ve(r), wa(() => (e === 0 && (n = ec(() => t(() => $r(r)))), e += 1, () => {
      Ht(() => {
        e -= 1, e === 0 && (n?.(), n = void 0, $r(r));
      });
    })));
  };
}
var Tl = Xt | Dt;
function El(t, e, r, n) {
  new Ol(t, e, r, n);
}
class Ol {
  /** @type {Boundary | null} */
  parent;
  is_pending = !1;
  /**
   * API-level transformError transform function. Transforms errors before they reach the `failed` snippet.
   * Inherited from parent boundary, or defaults to identity.
   * @type {(error: unknown) => unknown}
   */
  transform_error;
  /** @type {TemplateNode} */
  #e;
  /** @type {TemplateNode | null} */
  #t = Q ? ee : null;
  /** @type {BoundaryProps} */
  #r;
  /** @type {((anchor: Node) => void)} */
  #a;
  /** @type {Effect} */
  #n;
  /** @type {Effect | null} */
  #i = null;
  /** @type {Effect | null} */
  #s = null;
  /** @type {Effect | null} */
  #l = null;
  /** @type {DocumentFragment | null} */
  #o = null;
  #p = 0;
  #c = 0;
  #u = !1;
  /** @type {Set<Effect>} */
  #f = /* @__PURE__ */ new Set();
  /** @type {Set<Effect>} */
  #g = /* @__PURE__ */ new Set();
  /**
   * A source containing the number of pending async deriveds/expressions.
   * Only created if `$effect.pending()` is used inside the boundary,
   * otherwise updating the source results in needless `Batch.ensure()`
   * calls followed by no-op flushes
   * @type {Source<number> | null}
   */
  #d = null;
  #b = Cl(() => (this.#d = Lr(this.#p), () => {
    this.#d = null;
  }));
  /**
   * @param {TemplateNode} node
   * @param {BoundaryProps} props
   * @param {((anchor: Node) => void)} children
   * @param {((error: unknown) => unknown) | undefined} [transform_error]
   */
  constructor(e, r, n, s) {
    this.#e = e, this.#r = r, this.#a = (i) => {
      var a = (
        /** @type {Effect} */
        D
      );
      a.b = this, a.f |= Gn, n(i);
    }, this.parent = /** @type {Effect} */
    D.b, this.transform_error = s ?? this.parent?.transform_error ?? ((i) => i), this.#n = ka(() => {
      if (Q) {
        const i = (
          /** @type {Comment} */
          this.#t
        );
        gs();
        const a = i.data === Yi;
        if (i.data.startsWith(Bs)) {
          const c = JSON.parse(i.data.slice(Bs.length));
          this.#y(c);
        } else a ? this.#w() : this.#v();
      } else
        this.#_();
    }, Tl), Q && (this.#e = ee);
  }
  #v() {
    try {
      this.#i = Ye(() => this.#a(this.#e));
    } catch (e) {
      this.error(e);
    }
  }
  /**
   * @param {unknown} error The deserialized error from the server's hydration comment
   */
  #y(e) {
    const r = this.#r.failed;
    r && (this.#l = Ye(() => {
      r(
        this.#e,
        () => e,
        () => () => {
        }
      );
    }));
  }
  #w() {
    const e = this.#r.pending;
    e && (this.is_pending = !0, this.#s = Ye(() => e(this.#e)), Ht(() => {
      var r = this.#o = document.createDocumentFragment(), n = St();
      r.append(n), this.#i = this.#A(() => Ye(() => this.#a(n))), this.#c === 0 && (this.#e.before(r), this.#o = null, Sr(
        /** @type {Effect} */
        this.#s,
        () => {
          this.#s = null;
        }
      ), this.#h(
        /** @type {Batch} */
        M
      ));
    }));
  }
  #_() {
    try {
      if (this.is_pending = this.has_pending_snippet(), this.#c = 0, this.#p = 0, this.#i = Ye(() => {
        this.#a(this.#e);
      }), this.#c > 0) {
        var e = this.#o = document.createDocumentFragment();
        Ta(this.#i, e);
        const r = (
          /** @type {(anchor: Node) => void} */
          this.#r.pending
        );
        this.#s = Ye(() => r(this.#e));
      } else
        this.#h(
          /** @type {Batch} */
          M
        );
    } catch (r) {
      this.error(r);
    }
  }
  /**
   * @param {Batch} batch
   */
  #h(e) {
    this.is_pending = !1, e.transfer_effects(this.#f, this.#g);
  }
  /**
   * Defer an effect inside a pending boundary until the boundary resolves
   * @param {Effect} effect
   */
  defer_effect(e) {
    la(e, this.#f, this.#g);
  }
  /**
   * Returns `false` if the effect exists inside a boundary whose pending snippet is shown
   * @returns {boolean}
   */
  is_rendered() {
    return !this.is_pending && (!this.parent || this.parent.is_rendered());
  }
  has_pending_snippet() {
    return !!this.#r.pending;
  }
  /**
   * @template T
   * @param {() => T} fn
   */
  #A(e) {
    var r = D, n = j, s = ke;
    Ve(this.#n), $e(this.#n), Qt(this.#n.ctx);
    try {
      return Ke.ensure(), e();
    } catch (i) {
      return aa(i), null;
    } finally {
      Ve(r), $e(n), Qt(s);
    }
  }
  /**
   * Updates the pending count associated with the currently visible pending snippet,
   * if any, such that we can replace the snippet with content once work is done
   * @param {1 | -1} d
   * @param {Batch} batch
   */
  #m(e, r) {
    if (!this.has_pending_snippet()) {
      this.parent && this.parent.#m(e, r);
      return;
    }
    this.#c += e, this.#c === 0 && (this.#h(r), this.#s && Sr(this.#s, () => {
      this.#s = null;
    }), this.#o && (this.#e.before(this.#o), this.#o = null));
  }
  /**
   * Update the source that powers `$effect.pending()` inside this boundary,
   * and controls when the current `pending` snippet (if any) is removed.
   * Do not call from inside the class
   * @param {1 | -1} d
   * @param {Batch} batch
   */
  update_pending_count(e, r) {
    this.#m(e, r), this.#p += e, !(!this.#d || this.#u) && (this.#u = !0, Ht(() => {
      this.#u = !1, this.#d && en(this.#d, this.#p);
    }));
  }
  get_effect_pending() {
    return this.#b(), ve(
      /** @type {Source<number>} */
      this.#d
    );
  }
  /** @param {unknown} error */
  error(e) {
    if (!this.#r.onerror && !this.#r.failed)
      throw e;
    M?.is_fork ? (this.#i && M.skip_effect(this.#i), this.#s && M.skip_effect(this.#s), this.#l && M.skip_effect(this.#l), M.oncommit(() => {
      this.#k(e);
    })) : this.#k(e);
  }
  /**
   * @param {unknown} error
   */
  #k(e) {
    this.#i && (pe(this.#i), this.#i = null), this.#s && (pe(this.#s), this.#s = null), this.#l && (pe(this.#l), this.#l = null), Q && (Ue(
      /** @type {TemplateNode} */
      this.#t
    ), _l(), Ue(ra()));
    var r = this.#r.onerror;
    let n = this.#r.failed;
    var s = !1, i = !1;
    const a = () => {
      if (s) {
        yl();
        return;
      }
      s = !0, i && vl(), this.#l !== null && Sr(this.#l, () => {
        this.#l = null;
      }), this.#A(() => {
        this.#_();
      });
    }, l = (c) => {
      try {
        i = !0, r?.(c, a), i = !1;
      } catch (f) {
        at(f, this.#n && this.#n.parent);
      }
      n && (this.#l = this.#A(() => {
        try {
          return Ye(() => {
            var f = (
              /** @type {Effect} */
              D
            );
            f.b = this, f.f |= Gn, n(
              this.#e,
              () => c,
              () => a
            );
          });
        } catch (f) {
          return at(
            f,
            /** @type {Effect} */
            this.#n.parent
          ), null;
        }
      }));
    };
    Ht(() => {
      var c;
      try {
        c = this.transform_error(e);
      } catch (f) {
        at(f, this.#n && this.#n.parent);
        return;
      }
      c !== null && typeof c == "object" && typeof /** @type {any} */
      c.then == "function" ? c.then(
        l,
        /** @param {unknown} e */
        (f) => at(f, this.#n && this.#n.parent)
      ) : l(c);
    });
  }
}
function Pl(t, e, r, n) {
  const s = bs;
  var i = t.filter((b) => !b.settled), a = e.map(s);
  if (r.length === 0 && i.length === 0) {
    n(a);
    return;
  }
  var l = (
    /** @type {Effect} */
    D
  ), c = Dl(), f = i.length === 1 ? i[0].promise : i.length > 1 ? Promise.all(i.map((b) => b.promise)) : null;
  function d(b) {
    if ((l.f & we) === 0) {
      c();
      try {
        n([...a, ...b]);
      } catch (v) {
        at(v, l);
      }
      Qr();
    }
  }
  var u = ca();
  if (r.length === 0) {
    f.then(() => d([])).finally(u);
    return;
  }
  function o() {
    Promise.all(r.map((b) => /* @__PURE__ */ Nl(b))).then(d).catch((b) => at(b, l)).finally(u);
  }
  f ? f.then(() => {
    c(), o(), Qr();
  }) : o();
}
function Dl() {
  var t = (
    /** @type {Effect} */
    D
  ), e = j, r = ke, n = (
    /** @type {Batch} */
    M
  );
  return function(i = !0) {
    Ve(t), $e(e), Qt(r), i && (t.f & we) === 0 && (n?.activate(), n?.apply());
  };
}
function Qr(t = !0) {
  Ve(null), $e(null), Qt(null), t && M?.deactivate();
}
function ca() {
  var t = (
    /** @type {Effect} */
    D
  ), e = t.b, r = (
    /** @type {Batch} */
    M
  ), n = !!e?.is_rendered();
  return e?.update_pending_count(1, r), r.increment(n, t), () => {
    e?.update_pending_count(-1, r), r.decrement(n, t);
  };
}
// @__NO_SIDE_EFFECTS__
function bs(t) {
  var e = oe | ie;
  return D !== null && (D.f |= Dt), {
    ctx: ke,
    deps: null,
    effects: null,
    equals: na,
    f: e,
    fn: t,
    reactions: null,
    rv: 0,
    v: (
      /** @type {V} */
      ne
    ),
    wv: 0,
    parent: D,
    ac: null
  };
}
const br = Symbol("obsolete");
// @__NO_SIDE_EFFECTS__
function Nl(t, e, r) {
  let n = (
    /** @type {Effect | null} */
    D
  );
  n === null && dl();
  var s = (
    /** @type {Promise<V>} */
    /** @type {unknown} */
    void 0
  ), i = Lr(
    /** @type {V} */
    ne
  ), a = !j, l = /* @__PURE__ */ new Set();
  return Gl(() => {
    var c = (
      /** @type {Effect} */
      D
    ), f = Ki();
    s = f.promise;
    try {
      Promise.resolve(t()).then(f.resolve, (b) => {
        b !== vn && f.reject(b);
      }).finally(Qr);
    } catch (b) {
      f.reject(b), Qr();
    }
    var d = (
      /** @type {Batch} */
      M
    );
    if (a) {
      if ((c.f & Pt) !== 0)
        var u = ca();
      if (
        // boundary can be null if the async derived is inside an $effect.root not connected to the component render tree
        n.b?.is_rendered()
      )
        d.async_deriveds.get(c)?.reject(br);
      else
        for (const b of l.values())
          b.reject(br);
      l.add(f), d.async_deriveds.set(c, f);
    }
    const o = (b, v = void 0) => {
      u?.(), l.delete(f), v !== br && (d.activate(), v ? (i.f |= ot, en(i, v)) : ((i.f & ot) !== 0 && (i.f ^= ot), en(i, b)), d.deactivate());
    };
    f.promise.then(o, (b) => o(null, b || "unknown"));
  }), Wl(() => {
    for (const c of l)
      c.reject(br);
  }), new Promise((c) => {
    function f(d) {
      function u() {
        d === s ? c(i) : f(s);
      }
      d.then(u, u);
    }
    f(s);
  });
}
// @__NO_SIDE_EFFECTS__
function jl(t) {
  const e = /* @__PURE__ */ bs(t);
  return Ea(e), e;
}
function Rl(t) {
  var e = t.effects;
  if (e !== null) {
    t.effects = null;
    for (var r = 0; r < e.length; r += 1)
      pe(
        /** @type {Effect} */
        e[r]
      );
  }
}
function ys(t) {
  var e, r = D, n = t.parent;
  if (!ut && n !== null && t.v !== ne && // if it was never evaluated before, it's guaranteed to fail downstream, so we try to execute instead
  (n.f & (we | Ae)) !== 0)
    return bl(), t.v;
  Ve(n);
  try {
    t.f &= ~$t, Rl(t), e = Na(t);
  } finally {
    Ve(r);
  }
  return e;
}
function ua(t) {
  var e = ys(t);
  if (!t.equals(e) && (t.wv = Pa(), (!M?.is_fork || t.deps === null) && (M !== null ? (M.capture(t, e, !0), Kn?.capture(t, e, !0)) : t.v = e, t.deps === null))) {
    te(t, se);
    return;
  }
  ut || (Ne !== null ? (ks() || M?.is_fork) && Ne.set(t, e) : vs(t));
}
function Ll(t) {
  if (t.effects !== null)
    for (const e of t.effects)
      (e.teardown || e.ac) && (e.teardown?.(), e.ac?.abort(vn), e.fn !== null && (e.teardown = nl), e.ac = null, Or(e, 0), $s(e));
}
function da(t) {
  if (t.effects !== null)
    for (const e of t.effects)
      e.teardown && e.fn !== null && Kt(e);
}
let Rn = null, Rt = null, M = null, Kn = null, Ne = null, es = null, kr = !1, Ln = !1, Vt = null, Vr = null;
var Ys = 0;
let Ml = 1;
class Ke {
  id = Ml++;
  /** True as soon as `#process` was called */
  #e = !1;
  linked = !0;
  /** @type {Batch | null} */
  #t = null;
  /** @type {Batch | null} */
  #r = null;
  /** @type {Map<Effect, ReturnType<typeof deferred<any>>>} */
  async_deriveds = /* @__PURE__ */ new Map();
  /**
   * The current values of any signals that are updated in this batch.
   * Tuple format: [value, is_derived] (note: is_derived is false for deriveds, too, if they were overridden via assignment)
   * They keys of this map are identical to `this.#previous`
   * @type {Map<Value, [any, boolean]>}
   */
  current = /* @__PURE__ */ new Map();
  /**
   * The values of any signals (sources and deriveds) that are updated in this batch _before_ those updates took place.
   * They keys of this map are identical to `this.#current`
   * @type {Map<Value, any>}
   */
  previous = /* @__PURE__ */ new Map();
  /**
   * When the batch is committed (and the DOM is updated), we need to remove old branches
   * and append new ones by calling the functions added inside (if/each/key/etc) blocks
   * @type {Set<(batch: Batch) => void>}
   */
  #a = /* @__PURE__ */ new Set();
  /**
   * If a fork is discarded, we need to destroy any effects that are no longer needed
   * @type {Set<(batch: Batch) => void>}
   */
  #n = /* @__PURE__ */ new Set();
  /**
   * The number of async effects that are currently in flight
   */
  #i = 0;
  /**
   * Async effects that are currently in flight, _not_ inside a pending boundary
   * @type {Map<Effect, number>}
   */
  #s = /* @__PURE__ */ new Map();
  /**
   * A deferred that resolves when the batch is committed, used with `settled()`
   * TODO replace with Promise.withResolvers once supported widely enough
   * @type {{ promise: Promise<void>, resolve: (value?: any) => void, reject: (reason: unknown) => void } | null}
   */
  #l = null;
  /**
   * The root effects that need to be flushed
   * @type {Effect[]}
   */
  #o = [];
  /**
   * Effects created while this batch was active.
   * @type {Effect[]}
   */
  #p = [];
  /**
   * Deferred effects (which run after async work has completed) that are DIRTY
   * @type {Set<Effect>}
   */
  #c = /* @__PURE__ */ new Set();
  /**
   * Deferred effects that are MAYBE_DIRTY
   * @type {Set<Effect>}
   */
  #u = /* @__PURE__ */ new Set();
  /**
   * A map of branches that still exist, but will be destroyed when this batch
   * is committed — we skip over these during `process`.
   * The value contains child effects that were dirty/maybe_dirty before being reset,
   * so they can be rescheduled if the branch survives.
   * @type {Map<Effect, { d: Effect[], m: Effect[] }>}
   */
  #f = /* @__PURE__ */ new Map();
  /**
   * Inverse of #skipped_branches which we need to tell prior batches to unskip them when committing
   * @type {Set<Effect>}
   */
  #g = /* @__PURE__ */ new Set();
  is_fork = !1;
  #d = !1;
  constructor() {
    Rt === null ? Rn = Rt = this : (Rt.#r = this, this.#t = Rt), Rt = this;
  }
  #b() {
    if (this.is_fork) return !0;
    for (const n of this.#s.keys()) {
      for (var e = n, r = !1; e.parent !== null; ) {
        if (this.#f.has(e)) {
          r = !0;
          break;
        }
        e = e.parent;
      }
      if (!r)
        return !0;
    }
    return !1;
  }
  /**
   * Add an effect to the #skipped_branches map and reset its children
   * @param {Effect} effect
   */
  skip_effect(e) {
    this.#f.has(e) || this.#f.set(e, { d: [], m: [] }), this.#g.delete(e);
  }
  /**
   * Remove an effect from the #skipped_branches map and reschedule
   * any tracked dirty/maybe_dirty child effects
   * @param {Effect} effect
   * @param {(e: Effect) => void} callback
   */
  unskip_effect(e, r = (n) => this.schedule(n)) {
    var n = this.#f.get(e);
    if (n) {
      this.#f.delete(e);
      for (var s of n.d)
        te(s, ie), r(s);
      for (s of n.m)
        te(s, Ze), r(s);
    }
    this.#g.add(e);
  }
  #v() {
    this.#e = !0, Ys++ > 1e3 && (this.#m(), Fl());
    for (const c of this.#c)
      this.#u.delete(c), te(c, ie), this.schedule(c);
    for (const c of this.#u)
      te(c, Ze), this.schedule(c);
    const e = this.#o;
    this.#o = [], this.apply();
    var r = Vt = [], n = [], s = Vr = [];
    for (const c of e)
      try {
        this.#y(c, r, n);
      } catch (f) {
        throw pa(c), this.#b() || this.discard(), f;
      }
    if (M = null, s.length > 0) {
      var i = Ke.ensure();
      for (const c of s)
        i.schedule(c);
    }
    if (Vt = null, Vr = null, this.#b()) {
      this.#h(n), this.#h(r);
      for (const [c, f] of this.#f)
        ha(c, f);
      s.length > 0 && /** @type {unknown} */
      M.#v();
      return;
    }
    const a = this.#w();
    if (a) {
      this.#h(n), this.#h(r), a.#_(this);
      return;
    }
    this.#c.clear(), this.#u.clear();
    for (const c of this.#a) c(this);
    this.#a.clear(), Kn = this, Gs(n), Gs(r), Kn = null, this.#l?.resolve();
    var l = (
      /** @type {Batch | null} */
      /** @type {unknown} */
      M
    );
    if (this.#i === 0 && (this.#o.length === 0 || l !== null) && this.#m(), this.#o.length > 0)
      if (l !== null) {
        const c = l;
        c.#o.push(...this.#o.filter((f) => !c.#o.includes(f)));
      } else
        l = this;
    l !== null && l.#v();
  }
  /**
   * Traverse the effect tree, executing effects or stashing
   * them for later execution as appropriate
   * @param {Effect} root
   * @param {Effect[]} effects
   * @param {Effect[]} render_effects
   */
  #y(e, r, n) {
    e.f ^= se;
    for (var s = e.first; s !== null; ) {
      var i = s.f, a = (i & (Xe | Qe)) !== 0, l = a && (i & se) !== 0, c = l || (i & Ae) !== 0 || this.#f.has(s);
      if (!c && s.fn !== null) {
        a ? s.f ^= se : (i & Jt) !== 0 ? r.push(s) : Mr(s) && ((i & De) !== 0 && this.#u.add(s), Kt(s));
        var f = s.first;
        if (f !== null) {
          s = f;
          continue;
        }
      }
      for (; s !== null; ) {
        var d = s.next;
        if (d !== null) {
          s = d;
          break;
        }
        s = s.parent;
      }
    }
  }
  #w() {
    for (var e = this.#t; e !== null; ) {
      if (!e.is_fork) {
        for (const [r, [, n]] of this.current)
          if (e.current.has(r) && !n)
            return e;
      }
      e = e.#t;
    }
    return null;
  }
  /**
   * @param {Batch} batch
   */
  #_(e) {
    for (const [n, s] of e.current)
      !this.previous.has(n) && e.previous.has(n) && this.previous.set(n, e.previous.get(n)), this.current.set(n, s);
    for (const [n, s] of e.async_deriveds) {
      const i = this.async_deriveds.get(n);
      i && s.promise.then(i.resolve).catch(i.reject);
    }
    e.async_deriveds.clear(), this.transfer_effects(e.#c, e.#u);
    const r = (n) => {
      var s = n.reactions;
      if (s !== null)
        for (const l of s) {
          var i = l.f;
          if ((i & oe) !== 0)
            r(
              /** @type {Derived} */
              l
            );
          else {
            var a = (
              /** @type {Effect} */
              l
            );
            i & (qt | De) && !this.async_deriveds.has(a) && (this.#u.delete(a), te(a, ie), this.schedule(a));
          }
        }
    };
    for (const n of this.current.keys())
      r(n);
    this.oncommit(() => e.discard()), e.#m(), M = this, this.#v();
  }
  /**
   * @param {Effect[]} effects
   */
  #h(e) {
    for (var r = 0; r < e.length; r += 1)
      la(e[r], this.#c, this.#u);
  }
  /**
   * Associate a change to a given source with the current
   * batch, noting its previous and current values
   * @param {Value} source
   * @param {any} value
   * @param {boolean} [is_derived]
   */
  capture(e, r, n = !1) {
    e.v !== ne && !this.previous.has(e) && this.previous.set(e, e.v), (e.f & ot) === 0 && (this.current.set(e, [r, n]), Ne?.set(e, r)), this.is_fork || (e.v = r);
  }
  activate() {
    M = this;
  }
  deactivate() {
    M = null, Ne = null;
  }
  flush() {
    try {
      Ln = !0, M = this, this.#v();
    } finally {
      Ys = 0, es = null, Vt = null, Vr = null, Ln = !1, M = null, Ne = null, wt.clear();
    }
  }
  discard() {
    for (const e of this.#n) e(this);
    this.#n.clear();
    for (const e of this.async_deriveds.values())
      e.reject(br);
    this.#m(), this.#l?.resolve();
  }
  /**
   * @param {Effect} effect
   */
  register_created_effect(e) {
    this.#p.push(e);
  }
  #A() {
    for (let u = Rn; u !== null; u = u.#r) {
      var e = u.id < this.id, r = [];
      for (const [o, [b, v]] of this.current) {
        if (u.current.has(o)) {
          var n = (
            /** @type {[any, boolean]} */
            u.current.get(o)[0]
          );
          if (e && b !== n)
            u.current.set(o, [b, v]);
          else
            continue;
        }
        r.push(o);
      }
      if (e)
        for (const [o, b] of this.async_deriveds) {
          const v = u.async_deriveds.get(o);
          v && b.promise.then(v.resolve).catch(v.reject);
        }
      var s = [...u.current.keys()].filter(
        (o) => !/** @type {[any, boolean]} */
        u.current.get(o)[1]
      );
      if (!(!u.#e || s.length === 0)) {
        var i = s.filter((o) => !this.current.has(o));
        if (i.length === 0)
          e && u.discard();
        else if (r.length > 0) {
          if (e)
            for (const o of this.#g)
              u.unskip_effect(o, (b) => {
                (b.f & (De | qt)) !== 0 ? u.schedule(b) : u.#h([b]);
              });
          u.activate();
          var a = /* @__PURE__ */ new Set(), l = /* @__PURE__ */ new Map();
          for (var c of r)
            fa(c, i, a, l);
          l = /* @__PURE__ */ new Map();
          var f = [...u.current].filter(([o, b]) => {
            const v = this.current.get(o);
            return v ? v[0] !== b[0] || v[1] !== b[1] : !0;
          }).map(([o]) => o);
          if (f.length > 0)
            for (const o of this.#p)
              (o.f & (we | Ae | Jr)) === 0 && _s(o, f, l) && ((o.f & (qt | De)) !== 0 ? (te(o, ie), u.schedule(o)) : u.#c.add(o));
          if (u.#o.length > 0 && !u.#d) {
            u.apply();
            for (var d of u.#o)
              u.#y(d, [], []);
            u.#o = [];
          }
          u.deactivate();
        }
      }
    }
  }
  /**
   * @param {boolean} blocking
   * @param {Effect} effect
   */
  increment(e, r) {
    if (this.#i += 1, e) {
      let n = this.#s.get(r) ?? 0;
      this.#s.set(r, n + 1);
    }
  }
  /**
   * @param {boolean} blocking
   * @param {Effect} effect
   */
  decrement(e, r) {
    if (this.#i -= 1, e) {
      let n = this.#s.get(r) ?? 0;
      n === 1 ? this.#s.delete(r) : this.#s.set(r, n - 1);
    }
    this.#d || (this.#d = !0, Ht(() => {
      this.#d = !1, this.linked && this.flush();
    }));
  }
  /**
   * @param {Set<Effect>} dirty_effects
   * @param {Set<Effect>} maybe_dirty_effects
   */
  transfer_effects(e, r) {
    for (const n of e)
      this.#c.add(n);
    for (const n of r)
      this.#u.add(n);
    e.clear(), r.clear();
  }
  /** @param {(batch: Batch) => void} fn */
  oncommit(e) {
    this.#a.add(e);
  }
  /** @param {(batch: Batch) => void} fn */
  ondiscard(e) {
    this.#n.add(e);
  }
  settled() {
    return (this.#l ??= Ki()).promise;
  }
  static ensure() {
    if (M === null) {
      const e = M = new Ke();
      !Ln && !kr && Ht(() => {
        e.#e || e.flush();
      });
    }
    return M;
  }
  apply() {
    {
      Ne = null;
      return;
    }
  }
  /**
   *
   * @param {Effect} effect
   */
  schedule(e) {
    if (es = e, e.b?.is_pending && (e.f & (Jt | gn | ea)) !== 0 && (e.f & Pt) === 0) {
      e.b.defer_effect(e);
      return;
    }
    for (var r = e; r.parent !== null; ) {
      r = r.parent;
      var n = r.f;
      if (Vt !== null && r === D && (j === null || (j.f & oe) === 0))
        return;
      if ((n & (Qe | Xe)) !== 0) {
        if ((n & se) === 0)
          return;
        r.f ^= se;
      }
    }
    this.#o.push(r);
  }
  #m() {
    if (this.linked) {
      var e = this.#t, r = this.#r;
      e === null ? Rn = r : e.#r = r, r === null ? Rt = e : r.#t = e, this.linked = !1;
    }
  }
}
function h(t) {
  var e = kr;
  kr = !0;
  try {
    for (var r; ; ) {
      if (Sl(), M === null)
        return (
          /** @type {T} */
          r
        );
      M.flush();
    }
  } finally {
    kr = e;
  }
}
function Fl() {
  try {
    fl();
  } catch (t) {
    at(t, es);
  }
}
let He = null;
function Gs(t) {
  var e = t.length;
  if (e !== 0) {
    for (var r = 0; r < e; ) {
      var n = t[r++];
      if ((n.f & (we | Ae)) === 0 && Mr(n) && (He = /* @__PURE__ */ new Set(), Kt(n), n.deps === null && n.first === null && n.nodes === null && n.teardown === null && n.ac === null && Sa(n), He?.size > 0)) {
        wt.clear();
        for (const s of He) {
          if ((s.f & (we | Ae)) !== 0) continue;
          const i = [s];
          let a = s.parent;
          for (; a !== null; )
            He.has(a) && (He.delete(a), i.push(a)), a = a.parent;
          for (let l = i.length - 1; l >= 0; l--) {
            const c = i[l];
            (c.f & (we | Ae)) === 0 && Kt(c);
          }
        }
        He.clear();
      }
    }
    He = null;
  }
}
function fa(t, e, r, n) {
  if (!r.has(t) && (r.add(t), t.reactions !== null))
    for (const s of t.reactions) {
      const i = s.f;
      (i & oe) !== 0 ? fa(
        /** @type {Derived} */
        s,
        e,
        r,
        n
      ) : (i & (qt | De)) !== 0 && (i & ie) === 0 && _s(s, e, n) && (te(s, ie), As(
        /** @type {Effect} */
        s
      ));
    }
}
function _s(t, e, r) {
  const n = r.get(t);
  if (n !== void 0) return n;
  if (t.deps !== null)
    for (const s of t.deps) {
      if (Hr.call(e, s))
        return !0;
      if ((s.f & oe) !== 0 && _s(
        /** @type {Derived} */
        s,
        e,
        r
      ))
        return r.set(
          /** @type {Derived} */
          s,
          !0
        ), !0;
    }
  return r.set(t, !1), !1;
}
function As(t) {
  M.schedule(t);
}
function ha(t, e) {
  if (!((t.f & Xe) !== 0 && (t.f & se) !== 0)) {
    (t.f & ie) !== 0 ? e.d.push(t) : (t.f & Ze) !== 0 && e.m.push(t), te(t, se);
    for (var r = t.first; r !== null; )
      ha(r, e), r = r.next;
  }
}
function pa(t) {
  te(t, se);
  for (var e = t.first; e !== null; )
    pa(e), e = e.next;
}
let Kr = /* @__PURE__ */ new Set();
const wt = /* @__PURE__ */ new Map();
let ma = !1;
function Lr(t, e) {
  var r = {
    f: 0,
    // TODO ideally we could skip this altogether, but it causes type errors
    v: t,
    reactions: null,
    equals: na,
    rv: 0,
    wv: 0
  };
  return r;
}
// @__NO_SIDE_EFFECTS__
function rt(t, e) {
  const r = Lr(t);
  return Ea(r), r;
}
// @__NO_SIDE_EFFECTS__
function Il(t, e = !1, r = !0) {
  const n = Lr(t);
  return e || (n.equals = kl), n;
}
function Ge(t, e, r = !1) {
  j !== null && // since we are untracking the function inside `$inspect.with` we need to add this check
  // to ensure we error if state is set inside an inspect effect
  (!je || (j.f & Jr) !== 0) && sa() && (j.f & (oe | De | qt | Jr)) !== 0 && (Ie === null || !Ie.has(t)) && gl();
  let n = r ? yr(e) : e;
  return en(t, n, Vr);
}
function en(t, e, r = null) {
  if (!t.equals(e)) {
    wt.set(t, ut ? e : t.v);
    var n = Ke.ensure();
    if (n.capture(t, e), (t.f & oe) !== 0) {
      const s = (
        /** @type {Derived} */
        t
      );
      (t.f & ie) !== 0 && ys(s), Ne === null && vs(s);
    }
    t.wv = Pa(), ga(t, ie, r), D !== null && (D.f & se) !== 0 && (D.f & (Xe | Qe)) === 0 && (ye === null ? Ql([t]) : ye.push(t)), !n.is_fork && Kr.size > 0 && !ma && zl();
  }
  return e;
}
function zl() {
  ma = !1;
  for (const t of Kr) {
    (t.f & se) !== 0 && te(t, Ze);
    let e;
    try {
      e = Mr(t);
    } catch {
      e = !0;
    }
    e && Kt(t);
  }
  Kr.clear();
}
function $r(t) {
  Ge(t, t.v + 1);
}
function ga(t, e, r) {
  var n = t.reactions;
  if (n !== null)
    for (var s = n.length, i = 0; i < s; i++) {
      var a = n[i], l = a.f, c = (l & ie) === 0;
      if (c && te(a, e), (l & Jr) !== 0)
        Kr.add(
          /** @type {Effect} */
          a
        );
      else if ((l & oe) !== 0) {
        var f = (
          /** @type {Derived} */
          a
        );
        Ne?.delete(f), (l & $t) === 0 && (l & _e && (D === null || (D.f & Xr) === 0) && (a.f |= $t), ga(f, Ze, r));
      } else if (c) {
        var d = (
          /** @type {Effect} */
          a
        );
        (l & De) !== 0 && He !== null && He.add(d), r !== null ? r.push(d) : As(d);
      }
    }
}
function yr(t) {
  if (typeof t != "object" || t === null || jn in t)
    return t;
  const e = Qi(t);
  if (e !== tl && e !== rl)
    return t;
  var r = /* @__PURE__ */ new Map(), n = Xi(t), s = /* @__PURE__ */ rt(0), i = kt, a = (l) => {
    if (kt === i)
      return l();
    var c = j, f = kt;
    $e(null), Ks(i);
    var d = l();
    return $e(c), Ks(f), d;
  };
  return n && r.set("length", /* @__PURE__ */ rt(
    /** @type {any[]} */
    t.length
  )), new Proxy(
    /** @type {any} */
    t,
    {
      defineProperty(l, c, f) {
        (!("value" in f) || f.configurable === !1 || f.enumerable === !1 || f.writable === !1) && pl();
        var d = r.get(c);
        return d === void 0 ? a(() => {
          var u = /* @__PURE__ */ rt(f.value);
          return r.set(c, u), u;
        }) : Ge(d, f.value, !0), !0;
      },
      deleteProperty(l, c) {
        var f = r.get(c);
        if (f === void 0) {
          if (c in l) {
            const d = a(() => /* @__PURE__ */ rt(ne));
            r.set(c, d), $r(s);
          }
        } else
          Ge(f, ne), $r(s);
        return !0;
      },
      get(l, c, f) {
        if (c === jn)
          return t;
        var d = r.get(c), u = c in l;
        if (d === void 0 && (!u || Bt(l, c)?.writable) && (d = a(() => {
          var b = yr(u ? l[c] : ne), v = /* @__PURE__ */ rt(b);
          return v;
        }), r.set(c, d)), d !== void 0) {
          var o = ve(d);
          return o === ne ? void 0 : o;
        }
        return Reflect.get(l, c, f);
      },
      getOwnPropertyDescriptor(l, c) {
        var f = Reflect.getOwnPropertyDescriptor(l, c);
        if (f && "value" in f) {
          var d = r.get(c);
          d && (f.value = ve(d));
        } else if (f === void 0) {
          var u = r.get(c), o = u?.v;
          if (u !== void 0 && o !== ne)
            return {
              enumerable: !0,
              configurable: !0,
              value: o,
              writable: !0
            };
        }
        return f;
      },
      has(l, c) {
        if (c === jn)
          return !0;
        var f = r.get(c), d = f !== void 0 && f.v !== ne || Reflect.has(l, c);
        if (f !== void 0 || D !== null && (!d || Bt(l, c)?.writable)) {
          f === void 0 && (f = a(() => {
            var o = d ? yr(l[c]) : ne, b = /* @__PURE__ */ rt(o);
            return b;
          }), r.set(c, f));
          var u = ve(f);
          if (u === ne)
            return !1;
        }
        return d;
      },
      set(l, c, f, d) {
        var u = r.get(c), o = c in l;
        if (n && c === "length")
          for (var b = f; b < /** @type {Source<number>} */
          u.v; b += 1) {
            var v = r.get(b + "");
            v !== void 0 ? Ge(v, ne) : b in l && (v = a(() => /* @__PURE__ */ rt(ne)), r.set(b + "", v));
          }
        if (u === void 0)
          (!o || Bt(l, c)?.writable) && (u = a(() => /* @__PURE__ */ rt(void 0)), Ge(u, yr(f)), r.set(c, u));
        else {
          o = u.v !== ne;
          var g = a(() => yr(f));
          Ge(u, g);
        }
        var m = Reflect.getOwnPropertyDescriptor(l, c);
        if (m?.set && m.set.call(d, f), !o) {
          if (n && typeof c == "string") {
            var y = (
              /** @type {Source<number>} */
              r.get("length")
            ), fe = Number(c);
            Number.isInteger(fe) && fe >= y.v && Ge(y, fe + 1);
          }
          $r(s);
        }
        return !0;
      },
      ownKeys(l) {
        ve(s);
        var c = Reflect.ownKeys(l).filter((u) => {
          var o = r.get(u);
          return o === void 0 || o.v !== ne;
        });
        for (var [f, d] of r)
          d.v !== ne && !(f in l) && c.push(f);
        return c;
      },
      setPrototypeOf() {
        ml();
      }
    }
  );
}
var Js, va, ba, ya;
function ts() {
  if (Js === void 0) {
    Js = window, va = /Firefox/.test(navigator.userAgent);
    var t = Element.prototype, e = Node.prototype, r = Text.prototype;
    ba = Bt(e, "firstChild").get, ya = Bt(e, "nextSibling").get, qs(t) && (t[Jn] = void 0, t[ta] = null, t[Xn] = void 0, t.__e = void 0), qs(r) && (r[Qn] = void 0);
  }
}
function St(t = "") {
  return document.createTextNode(t);
}
// @__NO_SIDE_EFFECTS__
function tn(t) {
  return (
    /** @type {TemplateNode | null} */
    ba.call(t)
  );
}
// @__NO_SIDE_EFFECTS__
function mt(t) {
  return (
    /** @type {TemplateNode | null} */
    ya.call(t)
  );
}
function Ee(t, e) {
  if (!Q)
    return /* @__PURE__ */ tn(t);
  var r = /* @__PURE__ */ tn(ee);
  if (r === null)
    r = ee.appendChild(St());
  else if (e && r.nodeType !== ms) {
    var n = St();
    return r?.before(n), Ue(n), n;
  }
  return e && _a(
    /** @type {Text} */
    r
  ), Ue(r), r;
}
function qe(t, e = 1, r = !1) {
  let n = Q ? ee : t;
  for (var s; e--; )
    s = n, n = /** @type {TemplateNode} */
    /* @__PURE__ */ mt(n);
  if (!Q)
    return n;
  if (r) {
    if (n?.nodeType !== ms) {
      var i = St();
      return n === null ? s?.after(i) : n.before(i), Ue(i), i;
    }
    _a(
      /** @type {Text} */
      n
    );
  }
  return Ue(n), n;
}
function Zl(t) {
  t.textContent = "";
}
function Ul() {
  return !1;
}
function ws(t, e, r) {
  return (
    /** @type {T extends keyof HTMLElementTagNameMap ? HTMLElementTagNameMap[T] : Element} */
    document.createElement(t)
  );
}
function _a(t) {
  if (
    /** @type {string} */
    t.nodeValue.length < 65536
  )
    return;
  let e = t.nextSibling;
  for (; e !== null && e.nodeType === ms; )
    e.remove(), t.nodeValue += /** @type {string} */
    e.nodeValue, e = t.nextSibling;
}
function Aa(t) {
  var e = j, r = D;
  $e(null), Ve(null);
  try {
    return t();
  } finally {
    $e(e), Ve(r);
  }
}
function Vl(t, e) {
  var r = e.last;
  r === null ? e.last = e.first = t : (r.next = t, t.prev = r, e.last = t);
}
function Be(t, e) {
  var r = D;
  r !== null && (r.f & Ae) !== 0 && (t |= Ae);
  var n = {
    ctx: ke,
    deps: null,
    nodes: null,
    f: t | ie | _e,
    first: null,
    fn: e,
    last: null,
    next: null,
    parent: r,
    b: r && r.b,
    prev: null,
    teardown: null,
    wv: 0,
    ac: null
  };
  M?.register_created_effect(n);
  var s = n;
  if ((t & Jt) !== 0)
    Vt !== null ? Vt.push(n) : Ke.ensure().schedule(n);
  else if (e !== null) {
    try {
      Kt(n);
    } catch (a) {
      throw pe(n), a;
    }
    s.deps === null && s.teardown === null && s.nodes === null && s.first === s.last && // either `null`, or a singular child
    (s.f & Dt) === 0 && (s = s.first, (t & De) !== 0 && (t & Xt) !== 0 && s !== null && (s.f |= Xt));
  }
  if (s !== null && (s.parent = r, r !== null && Vl(s, r), j !== null && (j.f & oe) !== 0 && (t & Qe) === 0)) {
    var i = (
      /** @type {Derived} */
      j
    );
    (i.effects ??= []).push(s);
  }
  return n;
}
function ks() {
  return j !== null && !je;
}
function Wl(t) {
  const e = Be(gn, null);
  return te(e, se), e.teardown = t, e;
}
function Bl(t) {
  return Be(Jt | al, t);
}
function ql(t) {
  Ke.ensure();
  const e = Be(Qe | Dt, t);
  return () => {
    pe(e);
  };
}
function Hl(t) {
  Ke.ensure();
  const e = Be(Qe | Dt, t);
  return (r = {}) => new Promise((n) => {
    r.outro ? Sr(e, () => {
      pe(e), n(void 0);
    }) : (pe(e), n(void 0));
  });
}
function Yl(t) {
  return Be(Jt, t);
}
function Gl(t) {
  return Be(qt | Dt, t);
}
function wa(t, e = 0) {
  return Be(gn | e, t);
}
function Lt(t, e = [], r = [], n = []) {
  Pl(n, e, r, (s) => {
    Be(gn, () => {
      t(...s.map(ve));
    });
  });
}
function ka(t, e = 0) {
  var r = Be(De | e, t);
  return r;
}
function Ye(t) {
  return Be(Xe | Dt, t);
}
function $a(t) {
  var e = t.teardown;
  if (e !== null) {
    const r = ut, n = j;
    Qs(!0), $e(null);
    try {
      e.call(null);
    } finally {
      Qs(r), $e(n);
    }
  }
}
function $s(t, e = !1) {
  var r = t.first;
  for (t.first = t.last = null; r !== null; ) {
    const s = r.ac;
    s !== null && Aa(() => {
      s.abort(vn);
    });
    var n = r.next;
    (r.f & Qe) !== 0 ? r.parent = null : pe(r, e), r = n;
  }
}
function Jl(t) {
  for (var e = t.first; e !== null; ) {
    var r = e.next;
    (e.f & Xe) === 0 && pe(e), e = r;
  }
}
function pe(t, e = !0) {
  var r = !1;
  (e || (t.f & il) !== 0) && t.nodes !== null && t.nodes.end !== null && (Xl(
    t.nodes.start,
    /** @type {TemplateNode} */
    t.nodes.end
  ), r = !0), t.f |= Hs, $s(t, e && !r), Or(t, 0);
  var n = t.nodes && t.nodes.t;
  if (n !== null)
    for (const i of n)
      i.stop();
  $a(t), t.f ^= Hs, t.f |= we;
  var s = t.parent;
  s !== null && s.first !== null && Sa(t), t.next = t.prev = t.teardown = t.ctx = t.deps = t.fn = t.nodes = t.ac = t.b = null;
}
function Xl(t, e) {
  for (; t !== null; ) {
    var r = t === e ? null : /* @__PURE__ */ mt(t);
    t.remove(), t = r;
  }
}
function Sa(t) {
  var e = t.parent, r = t.prev, n = t.next;
  r !== null && (r.next = n), n !== null && (n.prev = r), e !== null && (e.first === t && (e.first = n), e.last === t && (e.last = r));
}
function Sr(t, e, r = !0) {
  var n = [];
  xa(t, n, !0);
  var s = () => {
    r && pe(t), e && e();
  }, i = n.length;
  if (i > 0) {
    var a = () => --i || s();
    for (var l of n)
      l.out(a);
  } else
    s();
}
function xa(t, e, r) {
  if ((t.f & Ae) === 0) {
    t.f ^= Ae;
    var n = t.nodes && t.nodes.t;
    if (n !== null)
      for (const l of n)
        (l.is_global || r) && e.push(l);
    for (var s = t.first; s !== null; ) {
      var i = s.next;
      if ((s.f & Qe) === 0) {
        var a = (s.f & Xt) !== 0 || // If this is a branch effect without a block effect parent,
        // it means the parent block effect was pruned. In that case,
        // transparency information was transferred to the branch effect.
        (s.f & Xe) !== 0 && (t.f & De) !== 0;
        xa(s, e, a ? r : !1);
      }
      s = i;
    }
  }
}
function Xs(t) {
  Ca(t, !0);
}
function Ca(t, e) {
  if ((t.f & Ae) !== 0) {
    t.f ^= Ae, (t.f & se) === 0 && (te(t, ie), Ke.ensure().schedule(t));
    for (var r = t.first; r !== null; ) {
      var n = r.next, s = (r.f & Xt) !== 0 || (r.f & Xe) !== 0;
      Ca(r, s ? e : !1), r = n;
    }
    var i = t.nodes && t.nodes.t;
    if (i !== null)
      for (const a of i)
        (a.is_global || e) && a.in();
  }
}
function Ta(t, e) {
  if (t.nodes)
    for (var r = t.nodes.start, n = t.nodes.end; r !== null; ) {
      var s = r === n ? null : /* @__PURE__ */ mt(r);
      e.append(r), r = s;
    }
}
let Wr = !1, ut = !1;
function Qs(t) {
  ut = t;
}
let j = null, je = !1;
function $e(t) {
  j = t;
}
let D = null;
function Ve(t) {
  D = t;
}
let Ie = null;
function Ea(t) {
  j !== null && (Ie ??= /* @__PURE__ */ new Set()).add(t);
}
let he = null, ge = 0, ye = null;
function Ql(t) {
  ye = t;
}
let Oa = 1, _t = 0, kt = _t;
function Ks(t) {
  kt = t;
}
function Pa() {
  return ++Oa;
}
function Mr(t) {
  var e = t.f;
  if ((e & ie) !== 0)
    return !0;
  if (e & oe && (t.f &= ~$t), (e & Ze) !== 0) {
    for (var r = (
      /** @type {Value[]} */
      t.deps
    ), n = r.length, s = 0; s < n; s++) {
      var i = r[s];
      if (Mr(
        /** @type {Derived} */
        i
      ) && ua(
        /** @type {Derived} */
        i
      ), i.wv > t.wv)
        return !0;
    }
    (e & _e) !== 0 && // During time traveling we don't want to reset the status so that
    // traversal of the graph in the other batches still happens
    Ne === null && te(t, se);
  }
  return !1;
}
function Da(t, e, r = !0) {
  var n = t.reactions;
  if (n !== null && !(Ie !== null && Ie.has(t)))
    for (var s = 0; s < n.length; s++) {
      var i = n[s];
      (i.f & oe) !== 0 ? Da(
        /** @type {Derived} */
        i,
        e,
        !1
      ) : e === i && (r ? te(i, ie) : (i.f & se) !== 0 && te(i, Ze), As(
        /** @type {Effect} */
        i
      ));
    }
}
function Na(t) {
  var e = he, r = ge, n = ye, s = j, i = Ie, a = ke, l = je, c = kt, f = t.f;
  he = /** @type {null | Value[]} */
  null, ge = 0, ye = null, j = (f & (Xe | Qe)) === 0 ? t : null, Ie = null, Qt(t.ctx), je = !1, kt = ++_t, t.ac !== null && (Aa(() => {
    t.ac.abort(vn);
  }), t.ac = null);
  try {
    t.f |= Xr;
    var d = (
      /** @type {Function} */
      t.fn
    ), u = d();
    t.f |= Pt;
    var o = t.deps, b = M?.is_fork;
    if (he !== null) {
      var v;
      if (b || Or(t, ge), o !== null && ge > 0)
        for (o.length = ge + he.length, v = 0; v < he.length; v++)
          o[ge + v] = he[v];
      else
        t.deps = o = he;
      if (ks() && (t.f & _e) !== 0)
        for (v = ge; v < o.length; v++)
          (o[v].reactions ??= []).push(t);
    } else !b && o !== null && ge < o.length && (Or(t, ge), o.length = ge);
    if (sa() && ye !== null && !je && o !== null && (t.f & (oe | Ze | ie)) === 0)
      for (v = 0; v < /** @type {Source[]} */
      ye.length; v++)
        Da(
          ye[v],
          /** @type {Effect} */
          t
        );
    if (s !== null && s !== t) {
      if (_t++, s.deps !== null)
        for (let g = 0; g < r; g += 1)
          s.deps[g].rv = _t;
      if (e !== null)
        for (const g of e)
          g.rv = _t;
      ye !== null && (n === null ? n = ye : n.push(.../** @type {Source[]} */
      ye));
    }
    return (t.f & ot) !== 0 && (t.f ^= ot), u;
  } catch (g) {
    return aa(g);
  } finally {
    t.f ^= Xr, he = e, ge = r, ye = n, j = s, Ie = i, Qt(a), je = l, kt = c;
  }
}
function Kl(t, e) {
  let r = e.reactions;
  if (r !== null) {
    var n = Qo.call(r, t);
    if (n !== -1) {
      var s = r.length - 1;
      s === 0 ? r = e.reactions = null : (r[n] = r[s], r.pop());
    }
  }
  if (r === null && (e.f & oe) !== 0 && // Destroying a child effect while updating a parent effect can cause a dependency to appear
  // to be unused, when in fact it is used by the currently-updating parent. Checking `new_deps`
  // allows us to skip the expensive work of disconnecting and immediately reconnecting it
  (he === null || !Hr.call(he, e))) {
    var i = (
      /** @type {Derived} */
      e
    );
    (i.f & _e) !== 0 && (i.f ^= _e, i.f &= ~$t), i.v !== ne && vs(i), Ll(i), Or(i, 0);
  }
}
function Or(t, e) {
  var r = t.deps;
  if (r !== null)
    for (var n = e; n < r.length; n++)
      Kl(t, r[n]);
}
function Kt(t) {
  var e = t.f;
  if ((e & we) === 0) {
    te(t, se);
    var r = D, n = Wr;
    D = t, Wr = !0;
    try {
      (e & (De | ea)) !== 0 ? Jl(t) : $s(t), $a(t);
      var s = Na(t);
      t.teardown = typeof s == "function" ? s : null, t.wv = Oa;
      var i;
      Ji && $l && (t.f & ie) !== 0 && t.deps;
    } finally {
      Wr = n, D = r;
    }
  }
}
function ve(t) {
  var e = t.f, r = (e & oe) !== 0;
  if (j !== null && !je) {
    var n = D !== null && (D.f & we) !== 0;
    if (!n && (Ie === null || !Ie.has(t))) {
      var s = j.deps;
      if ((j.f & Xr) !== 0)
        t.rv < _t && (t.rv = _t, he === null && s !== null && s[ge] === t ? ge++ : he === null ? he = [t] : he.push(t));
      else {
        j.deps ??= [], Hr.call(j.deps, t) || j.deps.push(t);
        var i = t.reactions;
        i === null ? t.reactions = [j] : Hr.call(i, j) || i.push(j);
      }
    }
  }
  if (ut && wt.has(t))
    return wt.get(t);
  if (r) {
    var a = (
      /** @type {Derived} */
      t
    );
    if (ut) {
      var l = a.v;
      return ((a.f & se) === 0 && a.reactions !== null || Ra(a)) && (l = ys(a)), wt.set(a, l), l;
    }
    var c = (a.f & _e) === 0 && !je && j !== null && (Wr || (j.f & _e) !== 0), f = (a.f & Pt) === 0;
    Mr(a) && (c && (a.f |= _e), ua(a)), c && !f && (da(a), ja(a));
  }
  if (Ne?.has(t))
    return Ne.get(t);
  if ((t.f & ot) !== 0)
    throw t.v;
  return t.v;
}
function ja(t) {
  if (t.f |= _e, t.deps !== null)
    for (const e of t.deps)
      (e.reactions ??= []).push(t), (e.f & oe) !== 0 && (e.f & _e) === 0 && (da(
        /** @type {Derived} */
        e
      ), ja(
        /** @type {Derived} */
        e
      ));
}
function Ra(t) {
  if (t.v === ne) return !0;
  if (t.deps === null) return !1;
  for (const e of t.deps)
    if (wt.has(e) || (e.f & oe) !== 0 && Ra(
      /** @type {Derived} */
      e
    ))
      return !0;
  return !1;
}
function ec(t) {
  var e = je;
  try {
    return je = !0, t();
  } finally {
    je = e;
  }
}
const _r = Symbol("events"), La = /* @__PURE__ */ new Set(), rs = /* @__PURE__ */ new Set();
function tc(t, e, r) {
  (e[_r] ??= {})[t] = r;
}
function rc(t) {
  for (var e = 0; e < t.length; e++)
    La.add(t[e]);
  for (var r of rs)
    r(t);
}
let ei = null;
function ti(t) {
  var e = this, r = (
    /** @type {Node} */
    e.ownerDocument
  ), n = t.type, s = t.composedPath?.() || [], i = (
    /** @type {null | Element} */
    s[0] || t.target
  );
  ei = t;
  var a = 0, l = ei === t && t[_r];
  if (l) {
    var c = s.indexOf(l);
    if (c !== -1 && (e === document || e === /** @type {any} */
    window)) {
      t[_r] = e;
      return;
    }
    var f = s.indexOf(e);
    if (f === -1)
      return;
    c <= f && (a = c);
  }
  if (i = /** @type {Element} */
  s[a] || t.target, i !== e) {
    Gr(t, "currentTarget", {
      configurable: !0,
      get() {
        return i || r;
      }
    });
    var d = j, u = D;
    $e(null), Ve(null);
    try {
      for (var o, b = []; i !== null && i !== e; ) {
        try {
          var v = i[_r]?.[n];
          v != null && (!/** @type {any} */
          i.disabled || // DOM could've been updated already by the time this is reached, so we check this as well
          // -> the target could not have been disabled because it emits the event in the first place
          t.target === i) && v.call(i, t);
        } catch (g) {
          o ? b.push(g) : o = g;
        }
        if (t.cancelBubble) break;
        a++, i = a < s.length ? (
          /** @type {Element} */
          s[a]
        ) : null;
      }
      if (o) {
        for (let g of b)
          queueMicrotask(() => {
            throw g;
          });
        throw o;
      }
    } finally {
      t[_r] = e, delete t.currentTarget, $e(d), Ve(u);
    }
  }
}
const nc = (
  // We gotta write it like this because after downleveling the pure comment may end up in the wrong location
  globalThis?.window?.trustedTypes && /* @__PURE__ */ globalThis.window.trustedTypes.createPolicy("svelte-trusted-html", {
    /** @param {string} html */
    createHTML: (t) => t
  })
);
function sc(t) {
  return (
    /** @type {string} */
    nc?.createHTML(t) ?? t
  );
}
function ic(t) {
  var e = ws("template");
  return e.innerHTML = sc(t.replaceAll("<!>", "<!---->")), e.content;
}
function ns(t, e) {
  var r = (
    /** @type {Effect} */
    D
  );
  r.nodes === null && (r.nodes = { start: t, end: e, a: null, t: null });
}
// @__NO_SIDE_EFFECTS__
function or(t, e) {
  var r = (e & Jo) !== 0, n, s = !t.startsWith("<!>");
  return () => {
    if (Q)
      return ns(ee, null), ee;
    n === void 0 && (n = ic(s ? t : "<!>" + t), n = /** @type {TemplateNode} */
    /* @__PURE__ */ tn(n));
    var i = (
      /** @type {TemplateNode} */
      r || va ? document.importNode(n, !0) : n.cloneNode(!0)
    );
    return ns(i, i), i;
  };
}
function vt(t, e) {
  if (Q) {
    var r = (
      /** @type {Effect & { nodes: EffectNodes }} */
      D
    );
    ((r.f & Pt) === 0 || r.nodes.end === null) && (r.nodes.end = ee), gs();
    return;
  }
  t !== null && t.before(
    /** @type {Node} */
    e
  );
}
const ac = ["touchstart", "touchmove"];
function oc(t) {
  return ac.includes(t);
}
function Mt(t, e) {
  var r = e == null ? "" : typeof e == "object" ? `${e}` : e;
  r !== /** @type {any} */
  (t[Qn] ??= t.nodeValue) && (t[Qn] = r, t.nodeValue = `${r}`);
}
function Ma(t, e) {
  return Fa(t, e);
}
function lc(t, e) {
  ts(), e.intro = e.intro ?? !1;
  const r = e.target, n = Q, s = ee;
  try {
    for (var i = /* @__PURE__ */ tn(r); i && (i.nodeType !== bn || /** @type {Comment} */
    i.data !== Hi); )
      i = /* @__PURE__ */ mt(i);
    if (!i)
      throw Gt;
    Ut(!0), Ue(
      /** @type {Comment} */
      i
    );
    const a = Fa(t, { ...e, anchor: i });
    return Ut(!1), /**  @type {Exports} */
    a;
  } catch (a) {
    if (a instanceof Error && a.message.split(`
`).some((l) => l.startsWith("https://svelte.dev/e/")))
      throw a;
    return a !== Gt && console.warn("Failed to hydrate: ", a), e.recover === !1 && hl(), ts(), Zl(r), Ut(!1), Ma(t, e);
  } finally {
    Ut(n), Ue(s);
  }
}
const Ur = /* @__PURE__ */ new Map();
function Fa(t, { target: e, anchor: r, props: n = {}, events: s, context: i, intro: a = !0, transformError: l }) {
  ts();
  var c = void 0, f = Hl(() => {
    var d = r ?? e.appendChild(St());
    El(
      /** @type {TemplateNode} */
      d,
      {
        pending: () => {
        }
      },
      (b) => {
        R({});
        var v = (
          /** @type {ComponentContext} */
          ke
        );
        if (i && (v.c = i), s && (n.$$events = s), Q && ns(
          /** @type {TemplateNode} */
          b,
          null
        ), c = t(b, n) || {}, Q && (D.nodes.end = ee, ee === null || ee.nodeType !== bn || /** @type {Comment} */
        ee.data !== Gi))
          throw yn(), Gt;
        L();
      },
      l
    );
    var u = /* @__PURE__ */ new Set(), o = (b) => {
      for (var v = 0; v < b.length; v++) {
        var g = b[v];
        if (!u.has(g)) {
          u.add(g);
          var m = oc(g);
          for (const jt of [e, document]) {
            var y = Ur.get(jt);
            y === void 0 && (y = /* @__PURE__ */ new Map(), Ur.set(jt, y));
            var fe = y.get(g);
            fe === void 0 ? (jt.addEventListener(g, ti, { passive: m }), y.set(g, 1)) : y.set(g, fe + 1);
          }
        }
      }
    };
    return o(Ko(La)), rs.add(o), () => {
      for (var b of u)
        for (const m of [e, document]) {
          var v = (
            /** @type {Map<string, number>} */
            Ur.get(m)
          ), g = (
            /** @type {number} */
            v.get(b)
          );
          --g == 0 ? (m.removeEventListener(b, ti), v.delete(b), v.size === 0 && Ur.delete(m)) : v.set(b, g);
        }
      rs.delete(o), d !== r && d.parentNode?.removeChild(d);
    };
  });
  return ss.set(c, f), c;
}
let ss = /* @__PURE__ */ new WeakMap();
function cc(t, e) {
  const r = ss.get(t);
  return r ? (ss.delete(t), r(e)) : Promise.resolve();
}
class uc {
  /** @type {TemplateNode} */
  anchor;
  /** @type {Map<Batch, Key>} */
  #e = /* @__PURE__ */ new Map();
  /**
   * Map of keys to effects that are currently rendered in the DOM.
   * These effects are visible and actively part of the document tree.
   * Example:
   * ```
   * {#if condition}
   * 	foo
   * {:else}
   * 	bar
   * {/if}
   * ```
   * Can result in the entries `true->Effect` and `false->Effect`
   * @type {Map<Key, Effect>}
   */
  #t = /* @__PURE__ */ new Map();
  /**
   * Similar to #onscreen with respect to the keys, but contains branches that are not yet
   * in the DOM, because their insertion is deferred.
   * @type {Map<Key, Branch>}
   */
  #r = /* @__PURE__ */ new Map();
  /**
   * Keys of effects that are currently outroing
   * @type {Set<Key>}
   */
  #a = /* @__PURE__ */ new Set();
  /**
   * Whether to pause (i.e. outro) on change, or destroy immediately.
   * This is necessary for `<svelte:element>`
   */
  #n = !0;
  /**
   * @param {TemplateNode} anchor
   * @param {boolean} transition
   */
  constructor(e, r = !0) {
    this.anchor = e, this.#n = r;
  }
  /**
   * @param {Batch} batch
   */
  #i = (e) => {
    if (this.#e.has(e)) {
      var r = (
        /** @type {Key} */
        this.#e.get(e)
      ), n = this.#t.get(r);
      if (n)
        Xs(n), this.#a.delete(r);
      else {
        var s = this.#r.get(r);
        s && (Xs(s.effect), this.#t.set(r, s.effect), this.#r.delete(r), s.fragment.lastChild.remove(), this.anchor.before(s.fragment), n = s.effect);
      }
      for (const [i, a] of this.#e) {
        if (this.#e.delete(i), i === e)
          break;
        const l = this.#r.get(a);
        l && (pe(l.effect), this.#r.delete(a));
      }
      for (const [i, a] of this.#t) {
        if (i === r || this.#a.has(i)) continue;
        const l = () => {
          if (Array.from(this.#e.values()).includes(i)) {
            var f = document.createDocumentFragment();
            Ta(a, f), f.append(St()), this.#r.set(i, { effect: a, fragment: f });
          } else
            pe(a);
          this.#a.delete(i), this.#t.delete(i);
        };
        this.#n || !n ? (this.#a.add(i), Sr(a, l, !1)) : l();
      }
    }
  };
  /**
   * @param {Batch} batch
   */
  #s = (e) => {
    this.#e.delete(e);
    const r = Array.from(this.#e.values());
    for (const [n, s] of this.#r)
      r.includes(n) || (pe(s.effect), this.#r.delete(n));
  };
  /**
   *
   * @param {any} key
   * @param {null | ((target: TemplateNode) => void)} fn
   */
  ensure(e, r) {
    var n = (
      /** @type {Batch} */
      M
    ), s = Ul();
    if (r && !this.#t.has(e) && !this.#r.has(e))
      if (s) {
        var i = document.createDocumentFragment(), a = St();
        i.append(a), this.#r.set(e, {
          effect: Ye(() => r(a)),
          fragment: i
        });
      } else
        this.#t.set(
          e,
          Ye(() => r(this.anchor))
        );
    if (this.#e.set(n, e), s) {
      for (const [l, c] of this.#t)
        l === e ? n.unskip_effect(c) : n.skip_effect(c);
      for (const [l, c] of this.#r)
        l === e ? n.unskip_effect(c.effect) : n.skip_effect(c.effect);
      n.oncommit(this.#i), n.ondiscard(this.#s);
    } else
      Q && (this.anchor = ee), this.#i(n);
  }
}
function dc(t, e, { bubbles: r = !1, cancelable: n = !1 } = {}) {
  return new CustomEvent(t, { detail: e, bubbles: r, cancelable: n });
}
function fc() {
  const t = ke;
  return t === null && ul(), (e, r, n) => {
    const s = (
      /** @type {Record<string, Function | Function[]>} */
      t.s.$$events?.[
        /** @type {string} */
        e
      ]
    );
    if (s) {
      const i = Xi(s) ? s.slice() : [s], a = dc(
        /** @type {string} */
        e,
        r,
        n
      );
      for (const l of i)
        l.call(t.x, a);
      return !a.defaultPrevented;
    }
    return !0;
  };
}
function fr(t, e, r = !1) {
  var n;
  Q && (n = ee, gs());
  var s = new uc(t), i = r ? Xt : 0;
  function a(l, c) {
    if (Q) {
      var f = Al(
        /** @type {TemplateNode} */
        n
      );
      if (l !== parseInt(f.substring(1))) {
        var d = ra();
        Ue(d), s.anchor = d, Ut(!1), s.ensure(l, c), Ut(!0);
        return;
      }
    }
    s.ensure(l, c);
  }
  ka(() => {
    var l = !1;
    e((c, f = 0) => {
      l = !0, a(f, c);
    }), l || a(-1, null);
  }, i);
}
function hc(t, e) {
  Yl(() => {
    var r = t.getRootNode(), n = (
      /** @type {ShadowRoot} */
      r.host ? (
        /** @type {ShadowRoot} */
        r
      ) : (
        /** @type {Document} */
        r.head ?? /** @type {Document} */
        r.ownerDocument.head
      )
    );
    if (!n.querySelector("#" + e.hash)) {
      const s = ws("style");
      s.id = e.hash, s.textContent = e.code, n.appendChild(s);
    }
  });
}
const ri = [...` 	
\r\f \v\uFEFF`];
function pc(t, e, r) {
  var n = "" + t;
  if (r) {
    for (var s of Object.keys(r))
      if (r[s])
        n = n ? n + " " + s : s;
      else if (n.length)
        for (var i = s.length, a = 0; (a = n.indexOf(s, a)) >= 0; ) {
          var l = a + i;
          (a === 0 || ri.includes(n[a - 1])) && (l === n.length || ri.includes(n[l])) ? n = (a === 0 ? "" : n.substring(0, a)) + n.substring(l + 1) : a = l;
        }
  }
  return n === "" ? null : n;
}
function ni(t, e = !1) {
  var r = e ? " !important;" : ";", n = "";
  for (var s of Object.keys(t)) {
    var i = t[s];
    i != null && i !== "" && (n += " " + s + ": " + i + r);
  }
  return n;
}
function mc(t, e) {
  if (e) {
    var r = "", n, s;
    return Array.isArray(e) ? (n = e[0], s = e[1]) : n = e, n && (r += ni(n)), s && (r += ni(s, !0)), r = r.trim(), r === "" ? null : r;
  }
  return String(t);
}
function gc(t, e, r, n, s, i) {
  var a = (
    /** @type {any} */
    t[Jn]
  );
  if (Q || a !== r || a === void 0) {
    var l = pc(r, n, i);
    (!Q || l !== t.getAttribute("class")) && (l == null ? t.removeAttribute("class") : t.className = l), t[Jn] = r;
  } else if (i && s !== i)
    for (var c in i) {
      var f = !!i[c];
      (s == null || f !== !!s[c]) && t.classList.toggle(c, f);
    }
  return i;
}
function Mn(t, e = {}, r, n) {
  for (var s in r) {
    var i = r[s];
    e[s] !== i && (r[s] == null ? t.style.removeProperty(s) : t.style.setProperty(s, i, n));
  }
}
function vc(t, e, r, n) {
  var s = (
    /** @type {any} */
    t[Xn]
  );
  if (Q || s !== e) {
    var i = mc(e, n);
    (!Q || i !== t.getAttribute("style")) && (i == null ? t.removeAttribute("style") : t.style.cssText = i), t[Xn] = e;
  } else n && (Array.isArray(n) ? (Mn(t, r?.[0], n[0]), Mn(t, r?.[1], n[1], "important")) : Mn(t, r, n));
  return n;
}
const bc = Symbol("is custom element"), yc = Symbol("is html"), _c = cl ? "link" : "LINK";
function Ft(t, e, r, n) {
  var s = Ac(t);
  Q && (s[e] = t.getAttribute(e), e === "src" || e === "srcset" || e === "href" && t.nodeName === _c) || s[e] !== (s[e] = r) && (e === "loading" && (t[ll] = r), r == null ? t.removeAttribute(e) : typeof r != "string" && wc(t).includes(e) ? t[e] = r : t.setAttribute(e, r));
}
function Ac(t) {
  return (
    /** @type {Record<string | symbol, unknown>} **/
    /** @type {any} */
    t[ta] ??= {
      [bc]: t.nodeName.includes("-"),
      [yc]: t.namespaceURI === Xo
    }
  );
}
var si = /* @__PURE__ */ new Map();
function wc(t) {
  var e = t.getAttribute("is") || t.nodeName, r = si.get(e);
  if (r) return r;
  si.set(e, r = []);
  for (var n, s = t, i = Element.prototype; i !== s; ) {
    n = el(s);
    for (var a in n)
      n[a].set && // better safe than sorry, we don't want spread attributes to mess with HTML content
      a !== "innerHTML" && a !== "textContent" && a !== "innerText" && r.push(a);
    s = Qi(s);
  }
  return r;
}
function p(t, e, r, n) {
  var s = (
    /** @type {V} */
    n
  ), i = !0, a = () => (i && (i = !1, s = /** @type {V} */
  n), s), l;
  l = /** @type {V} */
  t[e], l === void 0 && n !== void 0 && (l = a());
  var c;
  c = () => {
    var o = (
      /** @type {V} */
      t[e]
    );
    return o === void 0 ? a() : (i = !0, o);
  };
  var f = !1, d = /* @__PURE__ */ bs(() => (f = !1, c())), u = (
    /** @type {Effect} */
    D
  );
  return (
    /** @type {() => V} */
    (function(o, b) {
      if (arguments.length > 0) {
        const v = b ? ve(d) : o;
        return Ge(d, v), f = !0, s !== void 0 && (s = v), o;
      }
      return ut && f || (u.f & we) !== 0 ? d.v : ve(d);
    })
  );
}
function kc(t) {
  return new $c(t);
}
class $c {
  /** @type {any} */
  #e;
  /** @type {Record<string, any>} */
  #t;
  /**
   * @param {ComponentConstructorOptions & {
   *  component: any;
   * }} options
   */
  constructor(e) {
    var r = /* @__PURE__ */ new Map(), n = (i, a) => {
      var l = /* @__PURE__ */ Il(a, !1, !1);
      return r.set(i, l), l;
    };
    const s = new Proxy(
      { ...e.props || {}, $$events: {} },
      {
        get(i, a) {
          return ve(r.get(a) ?? n(a, Reflect.get(i, a)));
        },
        has(i, a) {
          return a === ol ? !0 : (ve(r.get(a) ?? n(a, Reflect.get(i, a))), Reflect.has(i, a));
        },
        set(i, a, l) {
          return Ge(r.get(a) ?? n(a, l), l), Reflect.set(i, a, l);
        }
      }
    );
    this.#t = (e.hydrate ? lc : Ma)(e.component, {
      target: e.target,
      anchor: e.anchor,
      props: s,
      context: e.context,
      intro: e.intro ?? !1,
      recover: e.recover,
      transformError: e.transformError
    }), (!e?.props?.$$host || e.sync === !1) && h(), this.#e = s.$$events;
    for (const i of Object.keys(this.#t))
      i === "$set" || i === "$destroy" || i === "$on" || Gr(this, i, {
        get() {
          return this.#t[i];
        },
        /** @param {any} value */
        set(a) {
          this.#t[i] = a;
        },
        enumerable: !0
      });
    this.#t.$set = /** @param {Record<string, any>} next */
    (i) => {
      Object.assign(s, i);
    }, this.#t.$destroy = () => {
      cc(this.#t);
    };
  }
  /** @param {Record<string, any>} props */
  $set(e) {
    this.#t.$set(e);
  }
  /**
   * @param {string} event
   * @param {(...args: any[]) => any} callback
   * @returns {any}
   */
  $on(e, r) {
    this.#e[e] = this.#e[e] || [];
    const n = (...s) => r.call(this, ...s);
    return this.#e[e].push(n), () => {
      this.#e[e] = this.#e[e].filter(
        /** @param {any} fn */
        (s) => s !== n
      );
    };
  }
  $destroy() {
    this.#t.$destroy();
  }
}
let Ia;
typeof HTMLElement == "function" && (Ia = class extends HTMLElement {
  /** The Svelte component constructor */
  $$ctor;
  /** Slots */
  $$s;
  /** @type {any} The Svelte component instance */
  $$c;
  /** Whether or not the custom element is connected */
  $$cn = !1;
  /** @type {Record<string, any>} Component props data */
  $$d = {};
  /** `true` if currently in the process of reflecting component props back to attributes */
  $$r = !1;
  /** @type {Record<string, CustomElementPropDefinition>} Props definition (name, reflected, type etc) */
  $$p_d = {};
  /** @type {Record<string, EventListenerOrEventListenerObject[]>} Event listeners */
  $$l = {};
  /** @type {Map<EventListenerOrEventListenerObject, Function>} Event listener unsubscribe functions */
  $$l_u = /* @__PURE__ */ new Map();
  /** @type {any} The managed render effect for reflecting attributes */
  $$me;
  /** @type {ShadowRoot | null} The ShadowRoot of the custom element */
  $$shadowRoot = null;
  /**
   * @param {*} $$componentCtor
   * @param {*} $$slots
   * @param {ShadowRootInit | undefined} shadow_root_init
   */
  constructor(t, e, r) {
    super(), this.$$ctor = t, this.$$s = e, r && (this.$$shadowRoot = this.attachShadow(r));
  }
  /**
   * @param {string} type
   * @param {EventListenerOrEventListenerObject} listener
   * @param {boolean | AddEventListenerOptions} [options]
   */
  addEventListener(t, e, r) {
    if (this.$$l[t] = this.$$l[t] || [], this.$$l[t].push(e), this.$$c) {
      const n = this.$$c.$on(t, e);
      this.$$l_u.set(e, n);
    }
    super.addEventListener(t, e, r);
  }
  /**
   * @param {string} type
   * @param {EventListenerOrEventListenerObject} listener
   * @param {boolean | AddEventListenerOptions} [options]
   */
  removeEventListener(t, e, r) {
    if (super.removeEventListener(t, e, r), this.$$c) {
      const n = this.$$l_u.get(e);
      n && (n(), this.$$l_u.delete(e));
    }
  }
  async connectedCallback() {
    if (this.$$cn = !0, !this.$$c) {
      let t = function(n) {
        return (s) => {
          const i = ws("slot");
          n !== "default" && (i.name = n), vt(s, i);
        };
      };
      if (await Promise.resolve(), !this.$$cn || this.$$c)
        return;
      const e = {}, r = Sc(this);
      for (const n of this.$$s)
        n in r && (n === "default" && !this.$$d.children ? (this.$$d.children = t(n), e.default = !0) : e[n] = t(n));
      for (const n of this.attributes) {
        const s = this.$$g_p(n.name);
        s in this.$$d || (this.$$d[s] = Br(s, n.value, this.$$p_d, "toProp"));
      }
      for (const n in this.$$p_d)
        !(n in this.$$d) && this[n] !== void 0 && (this.$$d[n] = this[n], delete this[n]);
      this.$$c = kc({
        component: this.$$ctor,
        target: this.$$shadowRoot || this,
        props: {
          ...this.$$d,
          $$slots: e,
          $$host: this
        }
      }), this.$$me = ql(() => {
        wa(() => {
          this.$$r = !0;
          for (const n of Yr(this.$$c)) {
            if (!this.$$p_d[n]?.reflect) continue;
            this.$$d[n] = this.$$c[n];
            const s = Br(
              n,
              this.$$d[n],
              this.$$p_d,
              "toAttribute"
            );
            s == null ? this.removeAttribute(this.$$p_d[n].attribute || n) : this.setAttribute(this.$$p_d[n].attribute || n, s);
          }
          this.$$r = !1;
        });
      });
      for (const n in this.$$l)
        for (const s of this.$$l[n]) {
          const i = this.$$c.$on(n, s);
          this.$$l_u.set(s, i);
        }
      this.$$l = {};
    }
  }
  // We don't need this when working within Svelte code, but for compatibility of people using this outside of Svelte
  // and setting attributes through setAttribute etc, this is helpful
  /**
   * @param {string} attr
   * @param {string} _oldValue
   * @param {string} newValue
   */
  attributeChangedCallback(t, e, r) {
    this.$$r || (t = this.$$g_p(t), this.$$d[t] = Br(t, r, this.$$p_d, "toProp"), this.$$c?.$set({ [t]: this.$$d[t] }));
  }
  disconnectedCallback() {
    this.$$cn = !1, Promise.resolve().then(() => {
      !this.$$cn && this.$$c && (this.$$c.$destroy(), this.$$me(), this.$$c = void 0);
    });
  }
  /**
   * @param {string} attribute_name
   */
  $$g_p(t) {
    return Yr(this.$$p_d).find(
      (e) => this.$$p_d[e].attribute === t || !this.$$p_d[e].attribute && e.toLowerCase() === t
    ) || t;
  }
});
function Br(t, e, r, n) {
  const s = r[t]?.type;
  if (e = s === "Boolean" && typeof e != "boolean" ? e != null : e, !n || !r[t])
    return e;
  if (n === "toAttribute")
    switch (s) {
      case "Object":
      case "Array":
        return e == null ? null : JSON.stringify(e);
      case "Boolean":
        return e ? "" : null;
      case "Number":
        return e ?? null;
      default:
        return e;
    }
  else
    switch (s) {
      case "Object":
      case "Array":
        return e && JSON.parse(e);
      case "Boolean":
        return e;
      // conversion already handled above
      case "Number":
        return e != null ? +e : e;
      default:
        return e;
    }
}
function Sc(t) {
  const e = {};
  return t.childNodes.forEach((r) => {
    e[
      /** @type {Element} node */
      r.slot || "default"
    ] = !0;
  }), e;
}
function I(t, e, r, n, s, i) {
  let a = class extends Ia {
    constructor() {
      super(t, r, s), this.$$p_d = e;
    }
    static get observedAttributes() {
      return Yr(e).map(
        (l) => (e[l].attribute || l).toLowerCase()
      );
    }
  };
  return Yr(e).forEach((l) => {
    Gr(a.prototype, l, {
      get() {
        return this.$$c && l in this.$$c ? this.$$c[l] : this.$$d[l];
      },
      set(c) {
        c = Br(l, c, e), this.$$d[l] = c;
        var f = this.$$c;
        if (f) {
          var d = Bt(f, l)?.get;
          d ? f[l] = c : f.$set({ [l]: c });
        }
      }
    });
  }), n.forEach((l) => {
    Gr(a.prototype, l, {
      get() {
        return this.$$c?.[l];
      }
    });
  }), t.element = /** @type {any} */
  a, a;
}
var xc = /* @__PURE__ */ or('<p class="svelte-136ik5h"> </p>'), Cc = /* @__PURE__ */ or('<div class="progress svelte-136ik5h" role="progressbar" aria-valuemin="0" aria-valuemax="100"><span class="svelte-136ik5h"></span></div>'), Tc = /* @__PURE__ */ or('<label class="svelte-136ik5h"><span class="svelte-136ik5h">Your response</span> <textarea rows="3" class="svelte-136ik5h"></textarea></label>'), Ec = /* @__PURE__ */ or('<button class="primary svelte-136ik5h" type="button"> </button>'), Oc = /* @__PURE__ */ or('<details class="svelte-136ik5h"><summary class="svelte-136ik5h">Advanced details</summary><pre class="svelte-136ik5h"> </pre></details>'), Pc = /* @__PURE__ */ or('<article data-projection="desktop tablet mobile terminal"><header class="svelte-136ik5h"><strong class="svelte-136ik5h"> </strong> <span class="status svelte-136ik5h" aria-live="polite"> </span></header> <!> <!> <!> <!> <!> <span class="terminal svelte-136ik5h" data-terminal-fallback=""> </span></article>'), Dc = {
  hash: "svelte-136ik5h",
  code: `:host {display:block;color:var(--focusa-fg, #172033);font:500 0.95rem/1.45 system-ui, sans-serif;}article.svelte-136ik5h {container-type:inline-size;display:grid;gap:.75rem;padding:1rem;border:1px solid var(--focusa-border, #c8d0dd);border-radius:.75rem;background:var(--focusa-surface, #fff);}article.warning.svelte-136ik5h {border-inline-start:.35rem solid #9b6500;}article.recovery.svelte-136ik5h {border-inline-start:.35rem solid #a32929;}article.shell.svelte-136ik5h {min-height:8rem;}header.svelte-136ik5h {display:flex;align-items:baseline;justify-content:space-between;gap:1rem;}.status.svelte-136ik5h {color:var(--focusa-muted, #536075);font-size:.8rem;}p.svelte-136ik5h {margin:0;}label.svelte-136ik5h {display:grid;gap:.35rem;}textarea.svelte-136ik5h {min-height:4.5rem;padding:.6rem;border:1px solid #7a879b;border-radius:.4rem;font:inherit;}.primary.svelte-136ik5h {justify-self:start;min-height:2.75rem;padding:.65rem 1rem;border:0;border-radius:.45rem;color:#fff;background:#174ea6;font:inherit;font-weight:700;cursor:pointer;}.primary.svelte-136ik5h:focus-visible, textarea.svelte-136ik5h:focus-visible, summary.svelte-136ik5h:focus-visible {outline:3px solid #f4b400;outline-offset:3px;}.primary.svelte-136ik5h:disabled {cursor:not-allowed;opacity:.6;}.progress.svelte-136ik5h {height:.7rem;overflow:hidden;border-radius:999px;background:#d7deea;}.progress.svelte-136ik5h span:where(.svelte-136ik5h) {display:block;height:100%;background:#176b45;transition:width 160ms ease;}pre.svelte-136ik5h {overflow:auto;white-space:pre-wrap;}.terminal.svelte-136ik5h {position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap;}
  @container (max-width: 30rem) {header.svelte-136ik5h {align-items:start;flex-direction:column;gap:.25rem;}.primary.svelte-136ik5h {width:100%;} }
  @media (prefers-reduced-motion: reduce) {.svelte-136ik5h, .progress.svelte-136ik5h span:where(.svelte-136ik5h) {scroll-behavior:auto !important;transition:none !important; animation: none !important;} }
  @media (prefers-contrast: more) {article.svelte-136ik5h {border-width:2px;} }`
};
function z(t, e) {
  R(e, !0), hc(t, Dc);
  let r = p(e, "componentName"), n = p(e, "kind", 7, "card"), s = p(e, "label", 7, "Focusa"), i = p(e, "description", 7, ""), a = p(e, "status", 7, "ready"), l = p(e, "progress", 7, 0), c = p(e, "primaryActionLabel", 7, "Continue"), f = p(e, "actionAvailable", 7, !1), d = p(e, "disabled", 7, !1), u = p(e, "busy", 7, !1), o = p(e, "details", 7, ""), b = p(e, "invokeAction"), v = fc(), g = () => {
    b()?.(), v("focusa-action", { componentName: r() });
  }, m = /* @__PURE__ */ jl(() => Math.max(0, Math.min(100, Number(l()) || 0)));
  var y = {
    get componentName() {
      return r();
    },
    set componentName(E) {
      r(E), h();
    },
    get kind() {
      return n();
    },
    set kind(E = "card") {
      n(E), h();
    },
    get label() {
      return s();
    },
    set label(E = "Focusa") {
      s(E), h();
    },
    get description() {
      return i();
    },
    set description(E = "") {
      i(E), h();
    },
    get status() {
      return a();
    },
    set status(E = "ready") {
      a(E), h();
    },
    get progress() {
      return l();
    },
    set progress(E = 0) {
      l(E), h();
    },
    get primaryActionLabel() {
      return c();
    },
    set primaryActionLabel(E = "Continue") {
      c(E), h();
    },
    get actionAvailable() {
      return f();
    },
    set actionAvailable(E = !1) {
      f(E), h();
    },
    get disabled() {
      return d();
    },
    set disabled(E = !1) {
      d(E), h();
    },
    get busy() {
      return u();
    },
    set busy(E = !1) {
      u(E), h();
    },
    get details() {
      return o();
    },
    set details(E = "") {
      o(E), h();
    },
    get invokeAction() {
      return b();
    },
    set invokeAction(E) {
      b(E), h();
    }
  }, fe = Pc();
  let jt;
  var Dn = Ee(fe), Nn = Ee(Dn), Uo = Ee(Nn, !0);
  Te(Nn);
  var Fs = qe(Nn, 2), Vo = Ee(Fs, !0);
  Te(Fs), Te(Dn);
  var Is = qe(Dn, 2), Wo = (E) => {
    var X = xc(), Ce = Ee(X, !0);
    Te(X), Lt(() => Mt(Ce, i())), vt(E, X);
  };
  fr(Is, (E) => {
    i() && E(Wo);
  });
  var zs = qe(Is, 2), Bo = (E) => {
    var X = Cc(), Ce = Ee(X);
    let Zr;
    Te(X), Lt(() => {
      Ft(X, "aria-label", s()), Ft(X, "aria-valuenow", ve(m)), Zr = vc(Ce, "", Zr, { width: `${ve(m)}%` });
    }), vt(E, X);
  };
  fr(zs, (E) => {
    n() === "progress" && E(Bo);
  });
  var Zs = qe(zs, 2), qo = (E) => {
    var X = Tc(), Ce = qe(Ee(X), 2);
    Te(X), Lt(() => {
      Ft(Ce, "aria-label", s()), Ce.disabled = d();
    }), vt(E, X);
  };
  fr(Zs, (E) => {
    n() === "input" && E(qo);
  });
  var Us = qe(Zs, 2), Ho = (E) => {
    var X = Ec(), Ce = Ee(X, !0);
    Te(X), Lt(() => {
      X.disabled = d() || u(), Mt(Ce, c());
    }), tc("click", X, g), vt(E, X);
  };
  fr(Us, (E) => {
    f() && E(Ho);
  });
  var Vs = qe(Us, 2), Yo = (E) => {
    var X = Oc(), Ce = qe(Ee(X)), Zr = Ee(Ce, !0);
    Te(Ce), Te(X), Lt(() => Mt(Zr, o())), vt(E, X);
  };
  fr(Vs, (E) => {
    o() && E(Yo);
  });
  var Ws = qe(Vs, 2), Go = Ee(Ws);
  return Te(Ws), Te(fe), Lt(() => {
    Ft(fe, "data-focusa-component", r()), Ft(fe, "role", n() === "recovery" ? "alert" : void 0), Ft(fe, "aria-busy", u()), jt = gc(fe, 1, "svelte-136ik5h", null, jt, {
      warning: n() === "warning",
      recovery: n() === "recovery",
      shell: n() === "shell"
    }), Mt(Uo, s()), Mt(Vo, u() ? "Saving…" : a()), Mt(Go, `${s() ?? ""}: ${a() ?? ""}`);
  }), vt(t, fe), L(y);
}
rc(["click"]), I(z, {
  componentName: {},
  kind: {},
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" });
function Nc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Stage Shell"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaStageShell",
    kind: "shell",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Stage Shell") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-stage-shell", I(Nc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function jc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Progress Stepper"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaProgressStepper",
    kind: "progress",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Progress Stepper") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-progress-stepper", I(jc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Rc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Primary Action"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaPrimaryAction",
    kind: "action",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Primary Action") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-primary-action", I(Rc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Lc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Next Step Card"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaNextStepCard",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Next Step Card") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-next-step-card", I(Lc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Mc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Source Connector Card"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaSourceConnectorCard",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Source Connector Card") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-source-connector-card", I(Mc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Fc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Dropzone"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaDropzone",
    kind: "input",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Dropzone") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-dropzone", I(Fc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Ic(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Import Scope Preview"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaImportScopePreview",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Import Scope Preview") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-import-scope-preview", I(Ic, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function zc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Context Summary"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaContextSummary",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Context Summary") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-context-summary", I(zc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Zc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Context Claim Review"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaContextClaimReview",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Context Claim Review") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-context-claim-review", I(Zc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Uc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Contradiction Card"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaContradictionCard",
    kind: "warning",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Contradiction Card") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-contradiction-card", I(Uc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Vc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Role Seed"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaRoleSeed",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Role Seed") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-role-seed", I(Vc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Wc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Role Draft"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaRoleDraft",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Role Draft") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-role-draft", I(Wc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Bc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Redline"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaRedline",
    kind: "warning",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Redline") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-redline", I(Bc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function qc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Grounding Sources"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaGroundingSources",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Grounding Sources") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-grounding-sources", I(qc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Hc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Question Card"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaQuestionCard",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Question Card") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-question-card", I(Hc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Yc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Recommendation Card"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaRecommendationCard",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Recommendation Card") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-recommendation-card", I(Yc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Gc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Answer Input"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaAnswerInput",
    kind: "input",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Answer Input") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-answer-input", I(Gc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Jc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Interview Branch Progress"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaInterviewBranchProgress",
    kind: "progress",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Interview Branch Progress") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-interview-branch-progress", I(Jc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Xc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Readiness Meter"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaReadinessMeter",
    kind: "progress",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Readiness Meter") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-readiness-meter", I(Xc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Qc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Spec Section Status"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaSpecSectionStatus",
    kind: "progress",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Spec Section Status") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-spec-section-status", I(Qc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function Kc(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Objection Card"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaObjectionCard",
    kind: "warning",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Objection Card") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-objection-card", I(Kc, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function eu(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Approval Card"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaApprovalCard",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Approval Card") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-approval-card", I(eu, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function tu(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Task Plan"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaTaskPlan",
    kind: "graph",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Task Plan") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-task-plan", I(tu, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function ru(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Dependency Graph"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaDependencyGraph",
    kind: "graph",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Dependency Graph") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-dependency-graph", I(ru, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function nu(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Provider Capability Card"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaProviderCapabilityCard",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Provider Capability Card") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-provider-capability-card", I(nu, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function su(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Workpoint Launch"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaWorkpointLaunch",
    kind: "action",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Workpoint Launch") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-workpoint-launch", I(su, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function iu(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Evidence Summary"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaEvidenceSummary",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Evidence Summary") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-evidence-summary", I(iu, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function au(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Receipt Card"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaReceiptCard",
    kind: "card",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Receipt Card") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-receipt-card", I(au, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function ou(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Recovery Card"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaRecoveryCard",
    kind: "recovery",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Recovery Card") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-recovery-card", I(ou, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function lu(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Advanced Details"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaAdvancedDetails",
    kind: "details",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Advanced Details") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-advanced-details", I(lu, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
function cu(t, e) {
  R(e, !0);
  let r = p(e, "label", 7, "Help Popover"), n = p(e, "description", 7, ""), s = p(e, "status", 7, "ready"), i = p(e, "progress", 7, 0), a = p(e, "primaryActionLabel", 7, "Continue"), l = p(e, "actionAvailable", 7, !1), c = p(e, "disabled", 7, !1), f = p(e, "busy", 7, !1), d = p(e, "details", 7, ""), u = p(e, "invokeAction", 7, void 0);
  return z(t, {
    componentName: "FocusaHelpPopover",
    kind: "details",
    get label() {
      return r();
    },
    get description() {
      return n();
    },
    get status() {
      return s();
    },
    get progress() {
      return i();
    },
    get primaryActionLabel() {
      return a();
    },
    get actionAvailable() {
      return l();
    },
    get disabled() {
      return c();
    },
    get busy() {
      return f();
    },
    get details() {
      return d();
    },
    get invokeAction() {
      return u();
    }
  }), L({
    get label() {
      return r();
    },
    set label(o = "Help Popover") {
      r(o), h();
    },
    get description() {
      return n();
    },
    set description(o = "") {
      n(o), h();
    },
    get status() {
      return s();
    },
    set status(o = "ready") {
      s(o), h();
    },
    get progress() {
      return i();
    },
    set progress(o = 0) {
      i(o), h();
    },
    get primaryActionLabel() {
      return a();
    },
    set primaryActionLabel(o = "Continue") {
      a(o), h();
    },
    get actionAvailable() {
      return l();
    },
    set actionAvailable(o = !1) {
      l(o), h();
    },
    get disabled() {
      return c();
    },
    set disabled(o = !1) {
      c(o), h();
    },
    get busy() {
      return f();
    },
    set busy(o = !1) {
      f(o), h();
    },
    get details() {
      return d();
    },
    set details(o = "") {
      d(o), h();
    },
    get invokeAction() {
      return u();
    },
    set invokeAction(o = void 0) {
      u(o), h();
    }
  });
}
customElements.define("focusa-help-popover", I(cu, {
  label: {},
  description: {},
  status: {},
  progress: {},
  primaryActionLabel: {},
  actionAvailable: {},
  disabled: {},
  busy: {},
  details: {},
  invokeAction: {}
}, [], [], { mode: "open" }));
var za = [
  {
    name: "FocusaStageShell",
    tag: "focusa-stage-shell",
    kind: "shell"
  },
  {
    name: "FocusaProgressStepper",
    tag: "focusa-progress-stepper",
    kind: "progress"
  },
  {
    name: "FocusaPrimaryAction",
    tag: "focusa-primary-action",
    kind: "action"
  },
  {
    name: "FocusaNextStepCard",
    tag: "focusa-next-step-card",
    kind: "card"
  },
  {
    name: "FocusaSourceConnectorCard",
    tag: "focusa-source-connector-card",
    kind: "card"
  },
  {
    name: "FocusaDropzone",
    tag: "focusa-dropzone",
    kind: "input"
  },
  {
    name: "FocusaImportScopePreview",
    tag: "focusa-import-scope-preview",
    kind: "card"
  },
  {
    name: "FocusaContextSummary",
    tag: "focusa-context-summary",
    kind: "card"
  },
  {
    name: "FocusaContextClaimReview",
    tag: "focusa-context-claim-review",
    kind: "card"
  },
  {
    name: "FocusaContradictionCard",
    tag: "focusa-contradiction-card",
    kind: "warning"
  },
  {
    name: "FocusaRoleSeed",
    tag: "focusa-role-seed",
    kind: "card"
  },
  {
    name: "FocusaRoleDraft",
    tag: "focusa-role-draft",
    kind: "card"
  },
  {
    name: "FocusaRedline",
    tag: "focusa-redline",
    kind: "warning"
  },
  {
    name: "FocusaGroundingSources",
    tag: "focusa-grounding-sources",
    kind: "card"
  },
  {
    name: "FocusaQuestionCard",
    tag: "focusa-question-card",
    kind: "card"
  },
  {
    name: "FocusaRecommendationCard",
    tag: "focusa-recommendation-card",
    kind: "card"
  },
  {
    name: "FocusaAnswerInput",
    tag: "focusa-answer-input",
    kind: "input"
  },
  {
    name: "FocusaInterviewBranchProgress",
    tag: "focusa-interview-branch-progress",
    kind: "progress"
  },
  {
    name: "FocusaReadinessMeter",
    tag: "focusa-readiness-meter",
    kind: "progress"
  },
  {
    name: "FocusaSpecSectionStatus",
    tag: "focusa-spec-section-status",
    kind: "progress"
  },
  {
    name: "FocusaObjectionCard",
    tag: "focusa-objection-card",
    kind: "warning"
  },
  {
    name: "FocusaApprovalCard",
    tag: "focusa-approval-card",
    kind: "card"
  },
  {
    name: "FocusaTaskPlan",
    tag: "focusa-task-plan",
    kind: "graph"
  },
  {
    name: "FocusaDependencyGraph",
    tag: "focusa-dependency-graph",
    kind: "graph"
  },
  {
    name: "FocusaProviderCapabilityCard",
    tag: "focusa-provider-capability-card",
    kind: "card"
  },
  {
    name: "FocusaWorkpointLaunch",
    tag: "focusa-workpoint-launch",
    kind: "action"
  },
  {
    name: "FocusaEvidenceSummary",
    tag: "focusa-evidence-summary",
    kind: "card"
  },
  {
    name: "FocusaReceiptCard",
    tag: "focusa-receipt-card",
    kind: "card"
  },
  {
    name: "FocusaRecoveryCard",
    tag: "focusa-recovery-card",
    kind: "recovery"
  },
  {
    name: "FocusaAdvancedDetails",
    tag: "focusa-advanced-details",
    kind: "details"
  },
  {
    name: "FocusaHelpPopover",
    tag: "focusa-help-popover",
    kind: "details"
  }
], F;
(function(t) {
  t.assertEqual = (s) => {
  };
  function e(s) {
  }
  t.assertIs = e;
  function r(s) {
    throw new Error();
  }
  t.assertNever = r, t.arrayToEnum = (s) => {
    const i = {};
    for (const a of s)
      i[a] = a;
    return i;
  }, t.getValidEnumValues = (s) => {
    const i = t.objectKeys(s).filter((l) => typeof s[s[l]] != "number"), a = {};
    for (const l of i)
      a[l] = s[l];
    return t.objectValues(a);
  }, t.objectValues = (s) => t.objectKeys(s).map(function(i) {
    return s[i];
  }), t.objectKeys = typeof Object.keys == "function" ? (s) => Object.keys(s) : (s) => {
    const i = [];
    for (const a in s)
      Object.prototype.hasOwnProperty.call(s, a) && i.push(a);
    return i;
  }, t.find = (s, i) => {
    for (const a of s)
      if (i(a))
        return a;
  }, t.isInteger = typeof Number.isInteger == "function" ? (s) => Number.isInteger(s) : (s) => typeof s == "number" && Number.isFinite(s) && Math.floor(s) === s;
  function n(s, i = " | ") {
    return s.map((a) => typeof a == "string" ? `'${a}'` : a).join(i);
  }
  t.joinValues = n, t.jsonStringifyReplacer = (s, i) => typeof i == "bigint" ? i.toString() : i;
})(F || (F = {}));
var ii;
(function(t) {
  t.mergeShapes = (e, r) => ({
    ...e,
    ...r
    // second overwrites first
  });
})(ii || (ii = {}));
const S = F.arrayToEnum([
  "string",
  "nan",
  "number",
  "integer",
  "float",
  "boolean",
  "date",
  "bigint",
  "symbol",
  "function",
  "undefined",
  "null",
  "array",
  "object",
  "unknown",
  "promise",
  "void",
  "never",
  "map",
  "set"
]), st = (t) => {
  switch (typeof t) {
    case "undefined":
      return S.undefined;
    case "string":
      return S.string;
    case "number":
      return Number.isNaN(t) ? S.nan : S.number;
    case "boolean":
      return S.boolean;
    case "function":
      return S.function;
    case "bigint":
      return S.bigint;
    case "symbol":
      return S.symbol;
    case "object":
      return Array.isArray(t) ? S.array : t === null ? S.null : t.then && typeof t.then == "function" && t.catch && typeof t.catch == "function" ? S.promise : typeof Map < "u" && t instanceof Map ? S.map : typeof Set < "u" && t instanceof Set ? S.set : typeof Date < "u" && t instanceof Date ? S.date : S.object;
    default:
      return S.unknown;
  }
}, _ = F.arrayToEnum([
  "invalid_type",
  "invalid_literal",
  "custom",
  "invalid_union",
  "invalid_union_discriminator",
  "invalid_enum_value",
  "unrecognized_keys",
  "invalid_arguments",
  "invalid_return_type",
  "invalid_date",
  "invalid_string",
  "too_small",
  "too_big",
  "invalid_intersection_types",
  "not_multiple_of",
  "not_finite"
]);
class Re extends Error {
  get errors() {
    return this.issues;
  }
  constructor(e) {
    super(), this.issues = [], this.addIssue = (n) => {
      this.issues = [...this.issues, n];
    }, this.addIssues = (n = []) => {
      this.issues = [...this.issues, ...n];
    };
    const r = new.target.prototype;
    Object.setPrototypeOf ? Object.setPrototypeOf(this, r) : this.__proto__ = r, this.name = "ZodError", this.issues = e;
  }
  format(e) {
    const r = e || function(i) {
      return i.message;
    }, n = { _errors: [] }, s = (i) => {
      for (const a of i.issues)
        if (a.code === "invalid_union")
          a.unionErrors.map(s);
        else if (a.code === "invalid_return_type")
          s(a.returnTypeError);
        else if (a.code === "invalid_arguments")
          s(a.argumentsError);
        else if (a.path.length === 0)
          n._errors.push(r(a));
        else {
          let l = n, c = 0;
          for (; c < a.path.length; ) {
            const f = a.path[c];
            c === a.path.length - 1 ? (l[f] = l[f] || { _errors: [] }, l[f]._errors.push(r(a))) : l[f] = l[f] || { _errors: [] }, l = l[f], c++;
          }
        }
    };
    return s(this), n;
  }
  static assert(e) {
    if (!(e instanceof Re))
      throw new Error(`Not a ZodError: ${e}`);
  }
  toString() {
    return this.message;
  }
  get message() {
    return JSON.stringify(this.issues, F.jsonStringifyReplacer, 2);
  }
  get isEmpty() {
    return this.issues.length === 0;
  }
  flatten(e = (r) => r.message) {
    const r = {}, n = [];
    for (const s of this.issues)
      if (s.path.length > 0) {
        const i = s.path[0];
        r[i] = r[i] || [], r[i].push(e(s));
      } else
        n.push(e(s));
    return { formErrors: n, fieldErrors: r };
  }
  get formErrors() {
    return this.flatten();
  }
}
Re.create = (t) => new Re(t);
const is = (t, e) => {
  let r;
  switch (t.code) {
    case _.invalid_type:
      t.received === S.undefined ? r = "Required" : r = `Expected ${t.expected}, received ${t.received}`;
      break;
    case _.invalid_literal:
      r = `Invalid literal value, expected ${JSON.stringify(t.expected, F.jsonStringifyReplacer)}`;
      break;
    case _.unrecognized_keys:
      r = `Unrecognized key(s) in object: ${F.joinValues(t.keys, ", ")}`;
      break;
    case _.invalid_union:
      r = "Invalid input";
      break;
    case _.invalid_union_discriminator:
      r = `Invalid discriminator value. Expected ${F.joinValues(t.options)}`;
      break;
    case _.invalid_enum_value:
      r = `Invalid enum value. Expected ${F.joinValues(t.options)}, received '${t.received}'`;
      break;
    case _.invalid_arguments:
      r = "Invalid function arguments";
      break;
    case _.invalid_return_type:
      r = "Invalid function return type";
      break;
    case _.invalid_date:
      r = "Invalid date";
      break;
    case _.invalid_string:
      typeof t.validation == "object" ? "includes" in t.validation ? (r = `Invalid input: must include "${t.validation.includes}"`, typeof t.validation.position == "number" && (r = `${r} at one or more positions greater than or equal to ${t.validation.position}`)) : "startsWith" in t.validation ? r = `Invalid input: must start with "${t.validation.startsWith}"` : "endsWith" in t.validation ? r = `Invalid input: must end with "${t.validation.endsWith}"` : F.assertNever(t.validation) : t.validation !== "regex" ? r = `Invalid ${t.validation}` : r = "Invalid";
      break;
    case _.too_small:
      t.type === "array" ? r = `Array must contain ${t.exact ? "exactly" : t.inclusive ? "at least" : "more than"} ${t.minimum} element(s)` : t.type === "string" ? r = `String must contain ${t.exact ? "exactly" : t.inclusive ? "at least" : "over"} ${t.minimum} character(s)` : t.type === "number" ? r = `Number must be ${t.exact ? "exactly equal to " : t.inclusive ? "greater than or equal to " : "greater than "}${t.minimum}` : t.type === "bigint" ? r = `Number must be ${t.exact ? "exactly equal to " : t.inclusive ? "greater than or equal to " : "greater than "}${t.minimum}` : t.type === "date" ? r = `Date must be ${t.exact ? "exactly equal to " : t.inclusive ? "greater than or equal to " : "greater than "}${new Date(Number(t.minimum))}` : r = "Invalid input";
      break;
    case _.too_big:
      t.type === "array" ? r = `Array must contain ${t.exact ? "exactly" : t.inclusive ? "at most" : "less than"} ${t.maximum} element(s)` : t.type === "string" ? r = `String must contain ${t.exact ? "exactly" : t.inclusive ? "at most" : "under"} ${t.maximum} character(s)` : t.type === "number" ? r = `Number must be ${t.exact ? "exactly" : t.inclusive ? "less than or equal to" : "less than"} ${t.maximum}` : t.type === "bigint" ? r = `BigInt must be ${t.exact ? "exactly" : t.inclusive ? "less than or equal to" : "less than"} ${t.maximum}` : t.type === "date" ? r = `Date must be ${t.exact ? "exactly" : t.inclusive ? "smaller than or equal to" : "smaller than"} ${new Date(Number(t.maximum))}` : r = "Invalid input";
      break;
    case _.custom:
      r = "Invalid input";
      break;
    case _.invalid_intersection_types:
      r = "Intersection results could not be merged";
      break;
    case _.not_multiple_of:
      r = `Number must be a multiple of ${t.multipleOf}`;
      break;
    case _.not_finite:
      r = "Number must be finite";
      break;
    default:
      r = e.defaultError, F.assertNever(t);
  }
  return { message: r };
};
let uu = is;
function du() {
  return uu;
}
const fu = (t) => {
  const { data: e, path: r, errorMaps: n, issueData: s } = t, i = [...r, ...s.path || []], a = {
    ...s,
    path: i
  };
  if (s.message !== void 0)
    return {
      ...s,
      path: i,
      message: s.message
    };
  let l = "";
  const c = n.filter((f) => !!f).slice().reverse();
  for (const f of c)
    l = f(a, { data: e, defaultError: l }).message;
  return {
    ...s,
    path: i,
    message: l
  };
};
function w(t, e) {
  const r = du(), n = fu({
    issueData: e,
    data: t.data,
    path: t.path,
    errorMaps: [
      t.common.contextualErrorMap,
      // contextual error map is first priority
      t.schemaErrorMap,
      // then schema-bound map if available
      r,
      // then global override map
      r === is ? void 0 : is
      // then global default map
    ].filter((s) => !!s)
  });
  t.common.issues.push(n);
}
class de {
  constructor() {
    this.value = "valid";
  }
  dirty() {
    this.value === "valid" && (this.value = "dirty");
  }
  abort() {
    this.value !== "aborted" && (this.value = "aborted");
  }
  static mergeArray(e, r) {
    const n = [];
    for (const s of r) {
      if (s.status === "aborted")
        return T;
      s.status === "dirty" && e.dirty(), n.push(s.value);
    }
    return { status: e.value, value: n };
  }
  static async mergeObjectAsync(e, r) {
    const n = [];
    for (const s of r) {
      const i = await s.key, a = await s.value;
      n.push({
        key: i,
        value: a
      });
    }
    return de.mergeObjectSync(e, n);
  }
  static mergeObjectSync(e, r) {
    const n = {};
    for (const s of r) {
      const { key: i, value: a } = s;
      if (i.status === "aborted" || a.status === "aborted")
        return T;
      i.status === "dirty" && e.dirty(), a.status === "dirty" && e.dirty(), i.value !== "__proto__" && (typeof a.value < "u" || s.alwaysSet) && (n[i.value] = a.value);
    }
    return { status: e.value, value: n };
  }
}
const T = Object.freeze({
  status: "aborted"
}), Ar = (t) => ({ status: "dirty", value: t }), xe = (t) => ({ status: "valid", value: t }), ai = (t) => t.status === "aborted", oi = (t) => t.status === "dirty", er = (t) => t.status === "valid", rn = (t) => typeof Promise < "u" && t instanceof Promise;
var x;
(function(t) {
  t.errToObj = (e) => typeof e == "string" ? { message: e } : e || {}, t.toString = (e) => typeof e == "string" ? e : e?.message;
})(x || (x = {}));
class We {
  constructor(e, r, n, s) {
    this._cachedPath = [], this.parent = e, this.data = r, this._path = n, this._key = s;
  }
  get path() {
    return this._cachedPath.length || (Array.isArray(this._key) ? this._cachedPath.push(...this._path, ...this._key) : this._cachedPath.push(...this._path, this._key)), this._cachedPath;
  }
}
const li = (t, e) => {
  if (er(e))
    return { success: !0, data: e.value };
  if (!t.common.issues.length)
    throw new Error("Validation failed but no issues detected.");
  return {
    success: !1,
    get error() {
      if (this._error)
        return this._error;
      const r = new Re(t.common.issues);
      return this._error = r, this._error;
    }
  };
};
function P(t) {
  if (!t)
    return {};
  const { errorMap: e, invalid_type_error: r, required_error: n, description: s } = t;
  if (e && (r || n))
    throw new Error(`Can't use "invalid_type_error" or "required_error" in conjunction with custom error map.`);
  return e ? { errorMap: e, description: s } : { errorMap: (a, l) => {
    const { message: c } = t;
    return a.code === "invalid_enum_value" ? { message: c ?? l.defaultError } : typeof l.data > "u" ? { message: c ?? n ?? l.defaultError } : a.code !== "invalid_type" ? { message: l.defaultError } : { message: c ?? r ?? l.defaultError };
  }, description: s };
}
class N {
  get description() {
    return this._def.description;
  }
  _getType(e) {
    return st(e.data);
  }
  _getOrReturnCtx(e, r) {
    return r || {
      common: e.parent.common,
      data: e.data,
      parsedType: st(e.data),
      schemaErrorMap: this._def.errorMap,
      path: e.path,
      parent: e.parent
    };
  }
  _processInputParams(e) {
    return {
      status: new de(),
      ctx: {
        common: e.parent.common,
        data: e.data,
        parsedType: st(e.data),
        schemaErrorMap: this._def.errorMap,
        path: e.path,
        parent: e.parent
      }
    };
  }
  _parseSync(e) {
    const r = this._parse(e);
    if (rn(r))
      throw new Error("Synchronous parse encountered promise.");
    return r;
  }
  _parseAsync(e) {
    const r = this._parse(e);
    return Promise.resolve(r);
  }
  parse(e, r) {
    const n = this.safeParse(e, r);
    if (n.success)
      return n.data;
    throw n.error;
  }
  safeParse(e, r) {
    const n = {
      common: {
        issues: [],
        async: r?.async ?? !1,
        contextualErrorMap: r?.errorMap
      },
      path: r?.path || [],
      schemaErrorMap: this._def.errorMap,
      parent: null,
      data: e,
      parsedType: st(e)
    }, s = this._parseSync({ data: e, path: n.path, parent: n });
    return li(n, s);
  }
  "~validate"(e) {
    const r = {
      common: {
        issues: [],
        async: !!this["~standard"].async
      },
      path: [],
      schemaErrorMap: this._def.errorMap,
      parent: null,
      data: e,
      parsedType: st(e)
    };
    if (!this["~standard"].async)
      try {
        const n = this._parseSync({ data: e, path: [], parent: r });
        return er(n) ? {
          value: n.value
        } : {
          issues: r.common.issues
        };
      } catch (n) {
        n?.message?.toLowerCase()?.includes("encountered") && (this["~standard"].async = !0), r.common = {
          issues: [],
          async: !0
        };
      }
    return this._parseAsync({ data: e, path: [], parent: r }).then((n) => er(n) ? {
      value: n.value
    } : {
      issues: r.common.issues
    });
  }
  async parseAsync(e, r) {
    const n = await this.safeParseAsync(e, r);
    if (n.success)
      return n.data;
    throw n.error;
  }
  async safeParseAsync(e, r) {
    const n = {
      common: {
        issues: [],
        contextualErrorMap: r?.errorMap,
        async: !0
      },
      path: r?.path || [],
      schemaErrorMap: this._def.errorMap,
      parent: null,
      data: e,
      parsedType: st(e)
    }, s = this._parse({ data: e, path: n.path, parent: n }), i = await (rn(s) ? s : Promise.resolve(s));
    return li(n, i);
  }
  refine(e, r) {
    const n = (s) => typeof r == "string" || typeof r > "u" ? { message: r } : typeof r == "function" ? r(s) : r;
    return this._refinement((s, i) => {
      const a = e(s), l = () => i.addIssue({
        code: _.custom,
        ...n(s)
      });
      return typeof Promise < "u" && a instanceof Promise ? a.then((c) => c ? !0 : (l(), !1)) : a ? !0 : (l(), !1);
    });
  }
  refinement(e, r) {
    return this._refinement((n, s) => e(n) ? !0 : (s.addIssue(typeof r == "function" ? r(n, s) : r), !1));
  }
  _refinement(e) {
    return new Et({
      schema: this,
      typeName: A.ZodEffects,
      effect: { type: "refinement", refinement: e }
    });
  }
  superRefine(e) {
    return this._refinement(e);
  }
  constructor(e) {
    this.spa = this.safeParseAsync, this._def = e, this.parse = this.parse.bind(this), this.safeParse = this.safeParse.bind(this), this.parseAsync = this.parseAsync.bind(this), this.safeParseAsync = this.safeParseAsync.bind(this), this.spa = this.spa.bind(this), this.refine = this.refine.bind(this), this.refinement = this.refinement.bind(this), this.superRefine = this.superRefine.bind(this), this.optional = this.optional.bind(this), this.nullable = this.nullable.bind(this), this.nullish = this.nullish.bind(this), this.array = this.array.bind(this), this.promise = this.promise.bind(this), this.or = this.or.bind(this), this.and = this.and.bind(this), this.transform = this.transform.bind(this), this.brand = this.brand.bind(this), this.default = this.default.bind(this), this.catch = this.catch.bind(this), this.describe = this.describe.bind(this), this.pipe = this.pipe.bind(this), this.readonly = this.readonly.bind(this), this.isNullable = this.isNullable.bind(this), this.isOptional = this.isOptional.bind(this), this["~standard"] = {
      version: 1,
      vendor: "zod",
      validate: (r) => this["~validate"](r)
    };
  }
  optional() {
    return lt.create(this, this._def);
  }
  nullable() {
    return nr.create(this, this._def);
  }
  nullish() {
    return this.nullable().optional();
  }
  array() {
    return ze.create(this);
  }
  promise() {
    return ln.create(this, this._def);
  }
  or(e) {
    return sn.create([this, e], this._def);
  }
  and(e) {
    return an.create(this, e, this._def);
  }
  transform(e) {
    return new Et({
      ...P(this._def),
      schema: this,
      typeName: A.ZodEffects,
      effect: { type: "transform", transform: e }
    });
  }
  default(e) {
    const r = typeof e == "function" ? e : () => e;
    return new cs({
      ...P(this._def),
      innerType: this,
      defaultValue: r,
      typeName: A.ZodDefault
    });
  }
  brand() {
    return new Ru({
      typeName: A.ZodBranded,
      type: this,
      ...P(this._def)
    });
  }
  catch(e) {
    const r = typeof e == "function" ? e : () => e;
    return new us({
      ...P(this._def),
      innerType: this,
      catchValue: r,
      typeName: A.ZodCatch
    });
  }
  describe(e) {
    const r = this.constructor;
    return new r({
      ...this._def,
      description: e
    });
  }
  pipe(e) {
    return Ss.create(this, e);
  }
  readonly() {
    return ds.create(this);
  }
  isOptional() {
    return this.safeParse(void 0).success;
  }
  isNullable() {
    return this.safeParse(null).success;
  }
}
const hu = /^c[^\s-]{8,}$/i, pu = /^[0-9a-z]+$/, mu = /^[0-9A-HJKMNP-TV-Z]{26}$/i, gu = /^[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12}$/i, vu = /^[a-z0-9_-]{21}$/i, bu = /^[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+\.[A-Za-z0-9-_]*$/, yu = /^[-+]?P(?!$)(?:(?:[-+]?\d+Y)|(?:[-+]?\d+[.,]\d+Y$))?(?:(?:[-+]?\d+M)|(?:[-+]?\d+[.,]\d+M$))?(?:(?:[-+]?\d+W)|(?:[-+]?\d+[.,]\d+W$))?(?:(?:[-+]?\d+D)|(?:[-+]?\d+[.,]\d+D$))?(?:T(?=[\d+-])(?:(?:[-+]?\d+H)|(?:[-+]?\d+[.,]\d+H$))?(?:(?:[-+]?\d+M)|(?:[-+]?\d+[.,]\d+M$))?(?:[-+]?\d+(?:[.,]\d+)?S)?)??$/, _u = /^(?!\.)(?!.*\.\.)([A-Z0-9_'+\-\.]*)[A-Z0-9_+-]@([A-Z0-9][A-Z0-9\-]*\.)+[A-Z]{2,}$/i, Au = "^(\\p{Extended_Pictographic}|\\p{Emoji_Component})+$";
let Fn;
const wu = /^(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])$/, ku = /^(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\/(3[0-2]|[12]?[0-9])$/, $u = /^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))$/, Su = /^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))\/(12[0-8]|1[01][0-9]|[1-9]?[0-9])$/, xu = /^([0-9a-zA-Z+/]{4})*(([0-9a-zA-Z+/]{2}==)|([0-9a-zA-Z+/]{3}=))?$/, Cu = /^([0-9a-zA-Z-_]{4})*(([0-9a-zA-Z-_]{2}(==)?)|([0-9a-zA-Z-_]{3}(=)?))?$/, Za = "((\\d\\d[2468][048]|\\d\\d[13579][26]|\\d\\d0[48]|[02468][048]00|[13579][26]00)-02-29|\\d{4}-((0[13578]|1[02])-(0[1-9]|[12]\\d|3[01])|(0[469]|11)-(0[1-9]|[12]\\d|30)|(02)-(0[1-9]|1\\d|2[0-8])))", Tu = new RegExp(`^${Za}$`);
function Ua(t) {
  let e = "[0-5]\\d";
  t.precision ? e = `${e}\\.\\d{${t.precision}}` : t.precision == null && (e = `${e}(\\.\\d+)?`);
  const r = t.precision ? "+" : "?";
  return `([01]\\d|2[0-3]):[0-5]\\d(:${e})${r}`;
}
function Eu(t) {
  return new RegExp(`^${Ua(t)}$`);
}
function Ou(t) {
  let e = `${Za}T${Ua(t)}`;
  const r = [];
  return r.push(t.local ? "Z?" : "Z"), t.offset && r.push("([+-]\\d{2}:?\\d{2})"), e = `${e}(${r.join("|")})`, new RegExp(`^${e}$`);
}
function Pu(t, e) {
  return !!((e === "v4" || !e) && wu.test(t) || (e === "v6" || !e) && $u.test(t));
}
function Du(t, e) {
  if (!bu.test(t))
    return !1;
  try {
    const [r] = t.split(".");
    if (!r)
      return !1;
    const n = r.replace(/-/g, "+").replace(/_/g, "/").padEnd(r.length + (4 - r.length % 4) % 4, "="), s = JSON.parse(atob(n));
    return !(typeof s != "object" || s === null || "typ" in s && s?.typ !== "JWT" || !s.alg || e && s.alg !== e);
  } catch {
    return !1;
  }
}
function Nu(t, e) {
  return !!((e === "v4" || !e) && ku.test(t) || (e === "v6" || !e) && Su.test(t));
}
class Fe extends N {
  _parse(e) {
    if (this._def.coerce && (e.data = String(e.data)), this._getType(e) !== S.string) {
      const i = this._getOrReturnCtx(e);
      return w(i, {
        code: _.invalid_type,
        expected: S.string,
        received: i.parsedType
      }), T;
    }
    const n = new de();
    let s;
    for (const i of this._def.checks)
      if (i.kind === "min")
        e.data.length < i.value && (s = this._getOrReturnCtx(e, s), w(s, {
          code: _.too_small,
          minimum: i.value,
          type: "string",
          inclusive: !0,
          exact: !1,
          message: i.message
        }), n.dirty());
      else if (i.kind === "max")
        e.data.length > i.value && (s = this._getOrReturnCtx(e, s), w(s, {
          code: _.too_big,
          maximum: i.value,
          type: "string",
          inclusive: !0,
          exact: !1,
          message: i.message
        }), n.dirty());
      else if (i.kind === "length") {
        const a = e.data.length > i.value, l = e.data.length < i.value;
        (a || l) && (s = this._getOrReturnCtx(e, s), a ? w(s, {
          code: _.too_big,
          maximum: i.value,
          type: "string",
          inclusive: !0,
          exact: !0,
          message: i.message
        }) : l && w(s, {
          code: _.too_small,
          minimum: i.value,
          type: "string",
          inclusive: !0,
          exact: !0,
          message: i.message
        }), n.dirty());
      } else if (i.kind === "email")
        _u.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
          validation: "email",
          code: _.invalid_string,
          message: i.message
        }), n.dirty());
      else if (i.kind === "emoji")
        Fn || (Fn = new RegExp(Au, "u")), Fn.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
          validation: "emoji",
          code: _.invalid_string,
          message: i.message
        }), n.dirty());
      else if (i.kind === "uuid")
        gu.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
          validation: "uuid",
          code: _.invalid_string,
          message: i.message
        }), n.dirty());
      else if (i.kind === "nanoid")
        vu.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
          validation: "nanoid",
          code: _.invalid_string,
          message: i.message
        }), n.dirty());
      else if (i.kind === "cuid")
        hu.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
          validation: "cuid",
          code: _.invalid_string,
          message: i.message
        }), n.dirty());
      else if (i.kind === "cuid2")
        pu.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
          validation: "cuid2",
          code: _.invalid_string,
          message: i.message
        }), n.dirty());
      else if (i.kind === "ulid")
        mu.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
          validation: "ulid",
          code: _.invalid_string,
          message: i.message
        }), n.dirty());
      else if (i.kind === "url")
        try {
          new URL(e.data);
        } catch {
          s = this._getOrReturnCtx(e, s), w(s, {
            validation: "url",
            code: _.invalid_string,
            message: i.message
          }), n.dirty();
        }
      else i.kind === "regex" ? (i.regex.lastIndex = 0, i.regex.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
        validation: "regex",
        code: _.invalid_string,
        message: i.message
      }), n.dirty())) : i.kind === "trim" ? e.data = e.data.trim() : i.kind === "includes" ? e.data.includes(i.value, i.position) || (s = this._getOrReturnCtx(e, s), w(s, {
        code: _.invalid_string,
        validation: { includes: i.value, position: i.position },
        message: i.message
      }), n.dirty()) : i.kind === "toLowerCase" ? e.data = e.data.toLowerCase() : i.kind === "toUpperCase" ? e.data = e.data.toUpperCase() : i.kind === "startsWith" ? e.data.startsWith(i.value) || (s = this._getOrReturnCtx(e, s), w(s, {
        code: _.invalid_string,
        validation: { startsWith: i.value },
        message: i.message
      }), n.dirty()) : i.kind === "endsWith" ? e.data.endsWith(i.value) || (s = this._getOrReturnCtx(e, s), w(s, {
        code: _.invalid_string,
        validation: { endsWith: i.value },
        message: i.message
      }), n.dirty()) : i.kind === "datetime" ? Ou(i).test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
        code: _.invalid_string,
        validation: "datetime",
        message: i.message
      }), n.dirty()) : i.kind === "date" ? Tu.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
        code: _.invalid_string,
        validation: "date",
        message: i.message
      }), n.dirty()) : i.kind === "time" ? Eu(i).test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
        code: _.invalid_string,
        validation: "time",
        message: i.message
      }), n.dirty()) : i.kind === "duration" ? yu.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
        validation: "duration",
        code: _.invalid_string,
        message: i.message
      }), n.dirty()) : i.kind === "ip" ? Pu(e.data, i.version) || (s = this._getOrReturnCtx(e, s), w(s, {
        validation: "ip",
        code: _.invalid_string,
        message: i.message
      }), n.dirty()) : i.kind === "jwt" ? Du(e.data, i.alg) || (s = this._getOrReturnCtx(e, s), w(s, {
        validation: "jwt",
        code: _.invalid_string,
        message: i.message
      }), n.dirty()) : i.kind === "cidr" ? Nu(e.data, i.version) || (s = this._getOrReturnCtx(e, s), w(s, {
        validation: "cidr",
        code: _.invalid_string,
        message: i.message
      }), n.dirty()) : i.kind === "base64" ? xu.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
        validation: "base64",
        code: _.invalid_string,
        message: i.message
      }), n.dirty()) : i.kind === "base64url" ? Cu.test(e.data) || (s = this._getOrReturnCtx(e, s), w(s, {
        validation: "base64url",
        code: _.invalid_string,
        message: i.message
      }), n.dirty()) : F.assertNever(i);
    return { status: n.value, value: e.data };
  }
  _regex(e, r, n) {
    return this.refinement((s) => e.test(s), {
      validation: r,
      code: _.invalid_string,
      ...x.errToObj(n)
    });
  }
  _addCheck(e) {
    return new Fe({
      ...this._def,
      checks: [...this._def.checks, e]
    });
  }
  email(e) {
    return this._addCheck({ kind: "email", ...x.errToObj(e) });
  }
  url(e) {
    return this._addCheck({ kind: "url", ...x.errToObj(e) });
  }
  emoji(e) {
    return this._addCheck({ kind: "emoji", ...x.errToObj(e) });
  }
  uuid(e) {
    return this._addCheck({ kind: "uuid", ...x.errToObj(e) });
  }
  nanoid(e) {
    return this._addCheck({ kind: "nanoid", ...x.errToObj(e) });
  }
  cuid(e) {
    return this._addCheck({ kind: "cuid", ...x.errToObj(e) });
  }
  cuid2(e) {
    return this._addCheck({ kind: "cuid2", ...x.errToObj(e) });
  }
  ulid(e) {
    return this._addCheck({ kind: "ulid", ...x.errToObj(e) });
  }
  base64(e) {
    return this._addCheck({ kind: "base64", ...x.errToObj(e) });
  }
  base64url(e) {
    return this._addCheck({
      kind: "base64url",
      ...x.errToObj(e)
    });
  }
  jwt(e) {
    return this._addCheck({ kind: "jwt", ...x.errToObj(e) });
  }
  ip(e) {
    return this._addCheck({ kind: "ip", ...x.errToObj(e) });
  }
  cidr(e) {
    return this._addCheck({ kind: "cidr", ...x.errToObj(e) });
  }
  datetime(e) {
    return typeof e == "string" ? this._addCheck({
      kind: "datetime",
      precision: null,
      offset: !1,
      local: !1,
      message: e
    }) : this._addCheck({
      kind: "datetime",
      precision: typeof e?.precision > "u" ? null : e?.precision,
      offset: e?.offset ?? !1,
      local: e?.local ?? !1,
      ...x.errToObj(e?.message)
    });
  }
  date(e) {
    return this._addCheck({ kind: "date", message: e });
  }
  time(e) {
    return typeof e == "string" ? this._addCheck({
      kind: "time",
      precision: null,
      message: e
    }) : this._addCheck({
      kind: "time",
      precision: typeof e?.precision > "u" ? null : e?.precision,
      ...x.errToObj(e?.message)
    });
  }
  duration(e) {
    return this._addCheck({ kind: "duration", ...x.errToObj(e) });
  }
  regex(e, r) {
    return this._addCheck({
      kind: "regex",
      regex: e,
      ...x.errToObj(r)
    });
  }
  includes(e, r) {
    return this._addCheck({
      kind: "includes",
      value: e,
      position: r?.position,
      ...x.errToObj(r?.message)
    });
  }
  startsWith(e, r) {
    return this._addCheck({
      kind: "startsWith",
      value: e,
      ...x.errToObj(r)
    });
  }
  endsWith(e, r) {
    return this._addCheck({
      kind: "endsWith",
      value: e,
      ...x.errToObj(r)
    });
  }
  min(e, r) {
    return this._addCheck({
      kind: "min",
      value: e,
      ...x.errToObj(r)
    });
  }
  max(e, r) {
    return this._addCheck({
      kind: "max",
      value: e,
      ...x.errToObj(r)
    });
  }
  length(e, r) {
    return this._addCheck({
      kind: "length",
      value: e,
      ...x.errToObj(r)
    });
  }
  /**
   * Equivalent to `.min(1)`
   */
  nonempty(e) {
    return this.min(1, x.errToObj(e));
  }
  trim() {
    return new Fe({
      ...this._def,
      checks: [...this._def.checks, { kind: "trim" }]
    });
  }
  toLowerCase() {
    return new Fe({
      ...this._def,
      checks: [...this._def.checks, { kind: "toLowerCase" }]
    });
  }
  toUpperCase() {
    return new Fe({
      ...this._def,
      checks: [...this._def.checks, { kind: "toUpperCase" }]
    });
  }
  get isDatetime() {
    return !!this._def.checks.find((e) => e.kind === "datetime");
  }
  get isDate() {
    return !!this._def.checks.find((e) => e.kind === "date");
  }
  get isTime() {
    return !!this._def.checks.find((e) => e.kind === "time");
  }
  get isDuration() {
    return !!this._def.checks.find((e) => e.kind === "duration");
  }
  get isEmail() {
    return !!this._def.checks.find((e) => e.kind === "email");
  }
  get isURL() {
    return !!this._def.checks.find((e) => e.kind === "url");
  }
  get isEmoji() {
    return !!this._def.checks.find((e) => e.kind === "emoji");
  }
  get isUUID() {
    return !!this._def.checks.find((e) => e.kind === "uuid");
  }
  get isNANOID() {
    return !!this._def.checks.find((e) => e.kind === "nanoid");
  }
  get isCUID() {
    return !!this._def.checks.find((e) => e.kind === "cuid");
  }
  get isCUID2() {
    return !!this._def.checks.find((e) => e.kind === "cuid2");
  }
  get isULID() {
    return !!this._def.checks.find((e) => e.kind === "ulid");
  }
  get isIP() {
    return !!this._def.checks.find((e) => e.kind === "ip");
  }
  get isCIDR() {
    return !!this._def.checks.find((e) => e.kind === "cidr");
  }
  get isBase64() {
    return !!this._def.checks.find((e) => e.kind === "base64");
  }
  get isBase64url() {
    return !!this._def.checks.find((e) => e.kind === "base64url");
  }
  get minLength() {
    let e = null;
    for (const r of this._def.checks)
      r.kind === "min" && (e === null || r.value > e) && (e = r.value);
    return e;
  }
  get maxLength() {
    let e = null;
    for (const r of this._def.checks)
      r.kind === "max" && (e === null || r.value < e) && (e = r.value);
    return e;
  }
}
Fe.create = (t) => new Fe({
  checks: [],
  typeName: A.ZodString,
  coerce: t?.coerce ?? !1,
  ...P(t)
});
function ju(t, e) {
  const r = (t.toString().split(".")[1] || "").length, n = (e.toString().split(".")[1] || "").length, s = r > n ? r : n, i = Number.parseInt(t.toFixed(s).replace(".", "")), a = Number.parseInt(e.toFixed(s).replace(".", ""));
  return i % a / 10 ** s;
}
class xt extends N {
  constructor() {
    super(...arguments), this.min = this.gte, this.max = this.lte, this.step = this.multipleOf;
  }
  _parse(e) {
    if (this._def.coerce && (e.data = Number(e.data)), this._getType(e) !== S.number) {
      const i = this._getOrReturnCtx(e);
      return w(i, {
        code: _.invalid_type,
        expected: S.number,
        received: i.parsedType
      }), T;
    }
    let n;
    const s = new de();
    for (const i of this._def.checks)
      i.kind === "int" ? F.isInteger(e.data) || (n = this._getOrReturnCtx(e, n), w(n, {
        code: _.invalid_type,
        expected: "integer",
        received: "float",
        message: i.message
      }), s.dirty()) : i.kind === "min" ? (i.inclusive ? e.data < i.value : e.data <= i.value) && (n = this._getOrReturnCtx(e, n), w(n, {
        code: _.too_small,
        minimum: i.value,
        type: "number",
        inclusive: i.inclusive,
        exact: !1,
        message: i.message
      }), s.dirty()) : i.kind === "max" ? (i.inclusive ? e.data > i.value : e.data >= i.value) && (n = this._getOrReturnCtx(e, n), w(n, {
        code: _.too_big,
        maximum: i.value,
        type: "number",
        inclusive: i.inclusive,
        exact: !1,
        message: i.message
      }), s.dirty()) : i.kind === "multipleOf" ? ju(e.data, i.value) !== 0 && (n = this._getOrReturnCtx(e, n), w(n, {
        code: _.not_multiple_of,
        multipleOf: i.value,
        message: i.message
      }), s.dirty()) : i.kind === "finite" ? Number.isFinite(e.data) || (n = this._getOrReturnCtx(e, n), w(n, {
        code: _.not_finite,
        message: i.message
      }), s.dirty()) : F.assertNever(i);
    return { status: s.value, value: e.data };
  }
  gte(e, r) {
    return this.setLimit("min", e, !0, x.toString(r));
  }
  gt(e, r) {
    return this.setLimit("min", e, !1, x.toString(r));
  }
  lte(e, r) {
    return this.setLimit("max", e, !0, x.toString(r));
  }
  lt(e, r) {
    return this.setLimit("max", e, !1, x.toString(r));
  }
  setLimit(e, r, n, s) {
    return new xt({
      ...this._def,
      checks: [
        ...this._def.checks,
        {
          kind: e,
          value: r,
          inclusive: n,
          message: x.toString(s)
        }
      ]
    });
  }
  _addCheck(e) {
    return new xt({
      ...this._def,
      checks: [...this._def.checks, e]
    });
  }
  int(e) {
    return this._addCheck({
      kind: "int",
      message: x.toString(e)
    });
  }
  positive(e) {
    return this._addCheck({
      kind: "min",
      value: 0,
      inclusive: !1,
      message: x.toString(e)
    });
  }
  negative(e) {
    return this._addCheck({
      kind: "max",
      value: 0,
      inclusive: !1,
      message: x.toString(e)
    });
  }
  nonpositive(e) {
    return this._addCheck({
      kind: "max",
      value: 0,
      inclusive: !0,
      message: x.toString(e)
    });
  }
  nonnegative(e) {
    return this._addCheck({
      kind: "min",
      value: 0,
      inclusive: !0,
      message: x.toString(e)
    });
  }
  multipleOf(e, r) {
    return this._addCheck({
      kind: "multipleOf",
      value: e,
      message: x.toString(r)
    });
  }
  finite(e) {
    return this._addCheck({
      kind: "finite",
      message: x.toString(e)
    });
  }
  safe(e) {
    return this._addCheck({
      kind: "min",
      inclusive: !0,
      value: Number.MIN_SAFE_INTEGER,
      message: x.toString(e)
    })._addCheck({
      kind: "max",
      inclusive: !0,
      value: Number.MAX_SAFE_INTEGER,
      message: x.toString(e)
    });
  }
  get minValue() {
    let e = null;
    for (const r of this._def.checks)
      r.kind === "min" && (e === null || r.value > e) && (e = r.value);
    return e;
  }
  get maxValue() {
    let e = null;
    for (const r of this._def.checks)
      r.kind === "max" && (e === null || r.value < e) && (e = r.value);
    return e;
  }
  get isInt() {
    return !!this._def.checks.find((e) => e.kind === "int" || e.kind === "multipleOf" && F.isInteger(e.value));
  }
  get isFinite() {
    let e = null, r = null;
    for (const n of this._def.checks) {
      if (n.kind === "finite" || n.kind === "int" || n.kind === "multipleOf")
        return !0;
      n.kind === "min" ? (r === null || n.value > r) && (r = n.value) : n.kind === "max" && (e === null || n.value < e) && (e = n.value);
    }
    return Number.isFinite(r) && Number.isFinite(e);
  }
}
xt.create = (t) => new xt({
  checks: [],
  typeName: A.ZodNumber,
  coerce: t?.coerce || !1,
  ...P(t)
});
class Ct extends N {
  constructor() {
    super(...arguments), this.min = this.gte, this.max = this.lte;
  }
  _parse(e) {
    if (this._def.coerce)
      try {
        e.data = BigInt(e.data);
      } catch {
        return this._getInvalidInput(e);
      }
    if (this._getType(e) !== S.bigint)
      return this._getInvalidInput(e);
    let n;
    const s = new de();
    for (const i of this._def.checks)
      i.kind === "min" ? (i.inclusive ? e.data < i.value : e.data <= i.value) && (n = this._getOrReturnCtx(e, n), w(n, {
        code: _.too_small,
        type: "bigint",
        minimum: i.value,
        inclusive: i.inclusive,
        message: i.message
      }), s.dirty()) : i.kind === "max" ? (i.inclusive ? e.data > i.value : e.data >= i.value) && (n = this._getOrReturnCtx(e, n), w(n, {
        code: _.too_big,
        type: "bigint",
        maximum: i.value,
        inclusive: i.inclusive,
        message: i.message
      }), s.dirty()) : i.kind === "multipleOf" ? e.data % i.value !== BigInt(0) && (n = this._getOrReturnCtx(e, n), w(n, {
        code: _.not_multiple_of,
        multipleOf: i.value,
        message: i.message
      }), s.dirty()) : F.assertNever(i);
    return { status: s.value, value: e.data };
  }
  _getInvalidInput(e) {
    const r = this._getOrReturnCtx(e);
    return w(r, {
      code: _.invalid_type,
      expected: S.bigint,
      received: r.parsedType
    }), T;
  }
  gte(e, r) {
    return this.setLimit("min", e, !0, x.toString(r));
  }
  gt(e, r) {
    return this.setLimit("min", e, !1, x.toString(r));
  }
  lte(e, r) {
    return this.setLimit("max", e, !0, x.toString(r));
  }
  lt(e, r) {
    return this.setLimit("max", e, !1, x.toString(r));
  }
  setLimit(e, r, n, s) {
    return new Ct({
      ...this._def,
      checks: [
        ...this._def.checks,
        {
          kind: e,
          value: r,
          inclusive: n,
          message: x.toString(s)
        }
      ]
    });
  }
  _addCheck(e) {
    return new Ct({
      ...this._def,
      checks: [...this._def.checks, e]
    });
  }
  positive(e) {
    return this._addCheck({
      kind: "min",
      value: BigInt(0),
      inclusive: !1,
      message: x.toString(e)
    });
  }
  negative(e) {
    return this._addCheck({
      kind: "max",
      value: BigInt(0),
      inclusive: !1,
      message: x.toString(e)
    });
  }
  nonpositive(e) {
    return this._addCheck({
      kind: "max",
      value: BigInt(0),
      inclusive: !0,
      message: x.toString(e)
    });
  }
  nonnegative(e) {
    return this._addCheck({
      kind: "min",
      value: BigInt(0),
      inclusive: !0,
      message: x.toString(e)
    });
  }
  multipleOf(e, r) {
    return this._addCheck({
      kind: "multipleOf",
      value: e,
      message: x.toString(r)
    });
  }
  get minValue() {
    let e = null;
    for (const r of this._def.checks)
      r.kind === "min" && (e === null || r.value > e) && (e = r.value);
    return e;
  }
  get maxValue() {
    let e = null;
    for (const r of this._def.checks)
      r.kind === "max" && (e === null || r.value < e) && (e = r.value);
    return e;
  }
}
Ct.create = (t) => new Ct({
  checks: [],
  typeName: A.ZodBigInt,
  coerce: t?.coerce ?? !1,
  ...P(t)
});
class nn extends N {
  _parse(e) {
    if (this._def.coerce && (e.data = !!e.data), this._getType(e) !== S.boolean) {
      const n = this._getOrReturnCtx(e);
      return w(n, {
        code: _.invalid_type,
        expected: S.boolean,
        received: n.parsedType
      }), T;
    }
    return xe(e.data);
  }
}
nn.create = (t) => new nn({
  typeName: A.ZodBoolean,
  coerce: t?.coerce || !1,
  ...P(t)
});
class tr extends N {
  _parse(e) {
    if (this._def.coerce && (e.data = new Date(e.data)), this._getType(e) !== S.date) {
      const i = this._getOrReturnCtx(e);
      return w(i, {
        code: _.invalid_type,
        expected: S.date,
        received: i.parsedType
      }), T;
    }
    if (Number.isNaN(e.data.getTime())) {
      const i = this._getOrReturnCtx(e);
      return w(i, {
        code: _.invalid_date
      }), T;
    }
    const n = new de();
    let s;
    for (const i of this._def.checks)
      i.kind === "min" ? e.data.getTime() < i.value && (s = this._getOrReturnCtx(e, s), w(s, {
        code: _.too_small,
        message: i.message,
        inclusive: !0,
        exact: !1,
        minimum: i.value,
        type: "date"
      }), n.dirty()) : i.kind === "max" ? e.data.getTime() > i.value && (s = this._getOrReturnCtx(e, s), w(s, {
        code: _.too_big,
        message: i.message,
        inclusive: !0,
        exact: !1,
        maximum: i.value,
        type: "date"
      }), n.dirty()) : F.assertNever(i);
    return {
      status: n.value,
      value: new Date(e.data.getTime())
    };
  }
  _addCheck(e) {
    return new tr({
      ...this._def,
      checks: [...this._def.checks, e]
    });
  }
  min(e, r) {
    return this._addCheck({
      kind: "min",
      value: e.getTime(),
      message: x.toString(r)
    });
  }
  max(e, r) {
    return this._addCheck({
      kind: "max",
      value: e.getTime(),
      message: x.toString(r)
    });
  }
  get minDate() {
    let e = null;
    for (const r of this._def.checks)
      r.kind === "min" && (e === null || r.value > e) && (e = r.value);
    return e != null ? new Date(e) : null;
  }
  get maxDate() {
    let e = null;
    for (const r of this._def.checks)
      r.kind === "max" && (e === null || r.value < e) && (e = r.value);
    return e != null ? new Date(e) : null;
  }
}
tr.create = (t) => new tr({
  checks: [],
  coerce: t?.coerce || !1,
  typeName: A.ZodDate,
  ...P(t)
});
class ci extends N {
  _parse(e) {
    if (this._getType(e) !== S.symbol) {
      const n = this._getOrReturnCtx(e);
      return w(n, {
        code: _.invalid_type,
        expected: S.symbol,
        received: n.parsedType
      }), T;
    }
    return xe(e.data);
  }
}
ci.create = (t) => new ci({
  typeName: A.ZodSymbol,
  ...P(t)
});
class ui extends N {
  _parse(e) {
    if (this._getType(e) !== S.undefined) {
      const n = this._getOrReturnCtx(e);
      return w(n, {
        code: _.invalid_type,
        expected: S.undefined,
        received: n.parsedType
      }), T;
    }
    return xe(e.data);
  }
}
ui.create = (t) => new ui({
  typeName: A.ZodUndefined,
  ...P(t)
});
class di extends N {
  _parse(e) {
    if (this._getType(e) !== S.null) {
      const n = this._getOrReturnCtx(e);
      return w(n, {
        code: _.invalid_type,
        expected: S.null,
        received: n.parsedType
      }), T;
    }
    return xe(e.data);
  }
}
di.create = (t) => new di({
  typeName: A.ZodNull,
  ...P(t)
});
class as extends N {
  constructor() {
    super(...arguments), this._any = !0;
  }
  _parse(e) {
    return xe(e.data);
  }
}
as.create = (t) => new as({
  typeName: A.ZodAny,
  ...P(t)
});
class fi extends N {
  constructor() {
    super(...arguments), this._unknown = !0;
  }
  _parse(e) {
    return xe(e.data);
  }
}
fi.create = (t) => new fi({
  typeName: A.ZodUnknown,
  ...P(t)
});
class dt extends N {
  _parse(e) {
    const r = this._getOrReturnCtx(e);
    return w(r, {
      code: _.invalid_type,
      expected: S.never,
      received: r.parsedType
    }), T;
  }
}
dt.create = (t) => new dt({
  typeName: A.ZodNever,
  ...P(t)
});
class hi extends N {
  _parse(e) {
    if (this._getType(e) !== S.undefined) {
      const n = this._getOrReturnCtx(e);
      return w(n, {
        code: _.invalid_type,
        expected: S.void,
        received: n.parsedType
      }), T;
    }
    return xe(e.data);
  }
}
hi.create = (t) => new hi({
  typeName: A.ZodVoid,
  ...P(t)
});
class ze extends N {
  _parse(e) {
    const { ctx: r, status: n } = this._processInputParams(e), s = this._def;
    if (r.parsedType !== S.array)
      return w(r, {
        code: _.invalid_type,
        expected: S.array,
        received: r.parsedType
      }), T;
    if (s.exactLength !== null) {
      const a = r.data.length > s.exactLength.value, l = r.data.length < s.exactLength.value;
      (a || l) && (w(r, {
        code: a ? _.too_big : _.too_small,
        minimum: l ? s.exactLength.value : void 0,
        maximum: a ? s.exactLength.value : void 0,
        type: "array",
        inclusive: !0,
        exact: !0,
        message: s.exactLength.message
      }), n.dirty());
    }
    if (s.minLength !== null && r.data.length < s.minLength.value && (w(r, {
      code: _.too_small,
      minimum: s.minLength.value,
      type: "array",
      inclusive: !0,
      exact: !1,
      message: s.minLength.message
    }), n.dirty()), s.maxLength !== null && r.data.length > s.maxLength.value && (w(r, {
      code: _.too_big,
      maximum: s.maxLength.value,
      type: "array",
      inclusive: !0,
      exact: !1,
      message: s.maxLength.message
    }), n.dirty()), r.common.async)
      return Promise.all([...r.data].map((a, l) => s.type._parseAsync(new We(r, a, r.path, l)))).then((a) => de.mergeArray(n, a));
    const i = [...r.data].map((a, l) => s.type._parseSync(new We(r, a, r.path, l)));
    return de.mergeArray(n, i);
  }
  get element() {
    return this._def.type;
  }
  min(e, r) {
    return new ze({
      ...this._def,
      minLength: { value: e, message: x.toString(r) }
    });
  }
  max(e, r) {
    return new ze({
      ...this._def,
      maxLength: { value: e, message: x.toString(r) }
    });
  }
  length(e, r) {
    return new ze({
      ...this._def,
      exactLength: { value: e, message: x.toString(r) }
    });
  }
  nonempty(e) {
    return this.min(1, e);
  }
}
ze.create = (t, e) => new ze({
  type: t,
  minLength: null,
  maxLength: null,
  exactLength: null,
  typeName: A.ZodArray,
  ...P(e)
});
function zt(t) {
  if (t instanceof re) {
    const e = {};
    for (const r in t.shape) {
      const n = t.shape[r];
      e[r] = lt.create(zt(n));
    }
    return new re({
      ...t._def,
      shape: () => e
    });
  } else return t instanceof ze ? new ze({
    ...t._def,
    type: zt(t.element)
  }) : t instanceof lt ? lt.create(zt(t.unwrap())) : t instanceof nr ? nr.create(zt(t.unwrap())) : t instanceof Tt ? Tt.create(t.items.map((e) => zt(e))) : t;
}
class re extends N {
  constructor() {
    super(...arguments), this._cached = null, this.nonstrict = this.passthrough, this.augment = this.extend;
  }
  _getCached() {
    if (this._cached !== null)
      return this._cached;
    const e = this._def.shape(), r = F.objectKeys(e);
    return this._cached = { shape: e, keys: r }, this._cached;
  }
  _parse(e) {
    if (this._getType(e) !== S.object) {
      const f = this._getOrReturnCtx(e);
      return w(f, {
        code: _.invalid_type,
        expected: S.object,
        received: f.parsedType
      }), T;
    }
    const { status: n, ctx: s } = this._processInputParams(e), { shape: i, keys: a } = this._getCached(), l = [];
    if (!(this._def.catchall instanceof dt && this._def.unknownKeys === "strip"))
      for (const f in s.data)
        a.includes(f) || l.push(f);
    const c = [];
    for (const f of a) {
      const d = i[f], u = s.data[f];
      c.push({
        key: { status: "valid", value: f },
        value: d._parse(new We(s, u, s.path, f)),
        alwaysSet: f in s.data
      });
    }
    if (this._def.catchall instanceof dt) {
      const f = this._def.unknownKeys;
      if (f === "passthrough")
        for (const d of l)
          c.push({
            key: { status: "valid", value: d },
            value: { status: "valid", value: s.data[d] }
          });
      else if (f === "strict")
        l.length > 0 && (w(s, {
          code: _.unrecognized_keys,
          keys: l
        }), n.dirty());
      else if (f !== "strip") throw new Error("Internal ZodObject error: invalid unknownKeys value.");
    } else {
      const f = this._def.catchall;
      for (const d of l) {
        const u = s.data[d];
        c.push({
          key: { status: "valid", value: d },
          value: f._parse(
            new We(s, u, s.path, d)
            //, ctx.child(key), value, getParsedType(value)
          ),
          alwaysSet: d in s.data
        });
      }
    }
    return s.common.async ? Promise.resolve().then(async () => {
      const f = [];
      for (const d of c) {
        const u = await d.key, o = await d.value;
        f.push({
          key: u,
          value: o,
          alwaysSet: d.alwaysSet
        });
      }
      return f;
    }).then((f) => de.mergeObjectSync(n, f)) : de.mergeObjectSync(n, c);
  }
  get shape() {
    return this._def.shape();
  }
  strict(e) {
    return x.errToObj, new re({
      ...this._def,
      unknownKeys: "strict",
      ...e !== void 0 ? {
        errorMap: (r, n) => {
          const s = this._def.errorMap?.(r, n).message ?? n.defaultError;
          return r.code === "unrecognized_keys" ? {
            message: x.errToObj(e).message ?? s
          } : {
            message: s
          };
        }
      } : {}
    });
  }
  strip() {
    return new re({
      ...this._def,
      unknownKeys: "strip"
    });
  }
  passthrough() {
    return new re({
      ...this._def,
      unknownKeys: "passthrough"
    });
  }
  // const AugmentFactory =
  //   <Def extends ZodObjectDef>(def: Def) =>
  //   <Augmentation extends ZodRawShape>(
  //     augmentation: Augmentation
  //   ): ZodObject<
  //     extendShape<ReturnType<Def["shape"]>, Augmentation>,
  //     Def["unknownKeys"],
  //     Def["catchall"]
  //   > => {
  //     return new ZodObject({
  //       ...def,
  //       shape: () => ({
  //         ...def.shape(),
  //         ...augmentation,
  //       }),
  //     }) as any;
  //   };
  extend(e) {
    return new re({
      ...this._def,
      shape: () => ({
        ...this._def.shape(),
        ...e
      })
    });
  }
  /**
   * Prior to zod@1.0.12 there was a bug in the
   * inferred type of merged objects. Please
   * upgrade if you are experiencing issues.
   */
  merge(e) {
    return new re({
      unknownKeys: e._def.unknownKeys,
      catchall: e._def.catchall,
      shape: () => ({
        ...this._def.shape(),
        ...e._def.shape()
      }),
      typeName: A.ZodObject
    });
  }
  // merge<
  //   Incoming extends AnyZodObject,
  //   Augmentation extends Incoming["shape"],
  //   NewOutput extends {
  //     [k in keyof Augmentation | keyof Output]: k extends keyof Augmentation
  //       ? Augmentation[k]["_output"]
  //       : k extends keyof Output
  //       ? Output[k]
  //       : never;
  //   },
  //   NewInput extends {
  //     [k in keyof Augmentation | keyof Input]: k extends keyof Augmentation
  //       ? Augmentation[k]["_input"]
  //       : k extends keyof Input
  //       ? Input[k]
  //       : never;
  //   }
  // >(
  //   merging: Incoming
  // ): ZodObject<
  //   extendShape<T, ReturnType<Incoming["_def"]["shape"]>>,
  //   Incoming["_def"]["unknownKeys"],
  //   Incoming["_def"]["catchall"],
  //   NewOutput,
  //   NewInput
  // > {
  //   const merged: any = new ZodObject({
  //     unknownKeys: merging._def.unknownKeys,
  //     catchall: merging._def.catchall,
  //     shape: () =>
  //       objectUtil.mergeShapes(this._def.shape(), merging._def.shape()),
  //     typeName: ZodFirstPartyTypeKind.ZodObject,
  //   }) as any;
  //   return merged;
  // }
  setKey(e, r) {
    return this.augment({ [e]: r });
  }
  // merge<Incoming extends AnyZodObject>(
  //   merging: Incoming
  // ): //ZodObject<T & Incoming["_shape"], UnknownKeys, Catchall> = (merging) => {
  // ZodObject<
  //   extendShape<T, ReturnType<Incoming["_def"]["shape"]>>,
  //   Incoming["_def"]["unknownKeys"],
  //   Incoming["_def"]["catchall"]
  // > {
  //   // const mergedShape = objectUtil.mergeShapes(
  //   //   this._def.shape(),
  //   //   merging._def.shape()
  //   // );
  //   const merged: any = new ZodObject({
  //     unknownKeys: merging._def.unknownKeys,
  //     catchall: merging._def.catchall,
  //     shape: () =>
  //       objectUtil.mergeShapes(this._def.shape(), merging._def.shape()),
  //     typeName: ZodFirstPartyTypeKind.ZodObject,
  //   }) as any;
  //   return merged;
  // }
  catchall(e) {
    return new re({
      ...this._def,
      catchall: e
    });
  }
  pick(e) {
    const r = {};
    for (const n of F.objectKeys(e))
      e[n] && this.shape[n] && (r[n] = this.shape[n]);
    return new re({
      ...this._def,
      shape: () => r
    });
  }
  omit(e) {
    const r = {};
    for (const n of F.objectKeys(this.shape))
      e[n] || (r[n] = this.shape[n]);
    return new re({
      ...this._def,
      shape: () => r
    });
  }
  /**
   * @deprecated
   */
  deepPartial() {
    return zt(this);
  }
  partial(e) {
    const r = {};
    for (const n of F.objectKeys(this.shape)) {
      const s = this.shape[n];
      e && !e[n] ? r[n] = s : r[n] = s.optional();
    }
    return new re({
      ...this._def,
      shape: () => r
    });
  }
  required(e) {
    const r = {};
    for (const n of F.objectKeys(this.shape))
      if (e && !e[n])
        r[n] = this.shape[n];
      else {
        let i = this.shape[n];
        for (; i instanceof lt; )
          i = i._def.innerType;
        r[n] = i;
      }
    return new re({
      ...this._def,
      shape: () => r
    });
  }
  keyof() {
    return Va(F.objectKeys(this.shape));
  }
}
re.create = (t, e) => new re({
  shape: () => t,
  unknownKeys: "strip",
  catchall: dt.create(),
  typeName: A.ZodObject,
  ...P(e)
});
re.strictCreate = (t, e) => new re({
  shape: () => t,
  unknownKeys: "strict",
  catchall: dt.create(),
  typeName: A.ZodObject,
  ...P(e)
});
re.lazycreate = (t, e) => new re({
  shape: t,
  unknownKeys: "strip",
  catchall: dt.create(),
  typeName: A.ZodObject,
  ...P(e)
});
class sn extends N {
  _parse(e) {
    const { ctx: r } = this._processInputParams(e), n = this._def.options;
    function s(i) {
      for (const l of i)
        if (l.result.status === "valid")
          return l.result;
      for (const l of i)
        if (l.result.status === "dirty")
          return r.common.issues.push(...l.ctx.common.issues), l.result;
      const a = i.map((l) => new Re(l.ctx.common.issues));
      return w(r, {
        code: _.invalid_union,
        unionErrors: a
      }), T;
    }
    if (r.common.async)
      return Promise.all(n.map(async (i) => {
        const a = {
          ...r,
          common: {
            ...r.common,
            issues: []
          },
          parent: null
        };
        return {
          result: await i._parseAsync({
            data: r.data,
            path: r.path,
            parent: a
          }),
          ctx: a
        };
      })).then(s);
    {
      let i;
      const a = [];
      for (const c of n) {
        const f = {
          ...r,
          common: {
            ...r.common,
            issues: []
          },
          parent: null
        }, d = c._parseSync({
          data: r.data,
          path: r.path,
          parent: f
        });
        if (d.status === "valid")
          return d;
        d.status === "dirty" && !i && (i = { result: d, ctx: f }), f.common.issues.length && a.push(f.common.issues);
      }
      if (i)
        return r.common.issues.push(...i.ctx.common.issues), i.result;
      const l = a.map((c) => new Re(c));
      return w(r, {
        code: _.invalid_union,
        unionErrors: l
      }), T;
    }
  }
  get options() {
    return this._def.options;
  }
}
sn.create = (t, e) => new sn({
  options: t,
  typeName: A.ZodUnion,
  ...P(e)
});
function os(t, e) {
  const r = st(t), n = st(e);
  if (t === e)
    return { valid: !0, data: t };
  if (r === S.object && n === S.object) {
    const s = F.objectKeys(e), i = F.objectKeys(t).filter((l) => s.indexOf(l) !== -1), a = { ...t, ...e };
    for (const l of i) {
      const c = os(t[l], e[l]);
      if (!c.valid)
        return { valid: !1 };
      a[l] = c.data;
    }
    return { valid: !0, data: a };
  } else if (r === S.array && n === S.array) {
    if (t.length !== e.length)
      return { valid: !1 };
    const s = [];
    for (let i = 0; i < t.length; i++) {
      const a = t[i], l = e[i], c = os(a, l);
      if (!c.valid)
        return { valid: !1 };
      s.push(c.data);
    }
    return { valid: !0, data: s };
  } else return r === S.date && n === S.date && +t == +e ? { valid: !0, data: t } : { valid: !1 };
}
class an extends N {
  _parse(e) {
    const { status: r, ctx: n } = this._processInputParams(e), s = (i, a) => {
      if (ai(i) || ai(a))
        return T;
      const l = os(i.value, a.value);
      return l.valid ? ((oi(i) || oi(a)) && r.dirty(), { status: r.value, value: l.data }) : (w(n, {
        code: _.invalid_intersection_types
      }), T);
    };
    return n.common.async ? Promise.all([
      this._def.left._parseAsync({
        data: n.data,
        path: n.path,
        parent: n
      }),
      this._def.right._parseAsync({
        data: n.data,
        path: n.path,
        parent: n
      })
    ]).then(([i, a]) => s(i, a)) : s(this._def.left._parseSync({
      data: n.data,
      path: n.path,
      parent: n
    }), this._def.right._parseSync({
      data: n.data,
      path: n.path,
      parent: n
    }));
  }
}
an.create = (t, e, r) => new an({
  left: t,
  right: e,
  typeName: A.ZodIntersection,
  ...P(r)
});
class Tt extends N {
  _parse(e) {
    const { status: r, ctx: n } = this._processInputParams(e);
    if (n.parsedType !== S.array)
      return w(n, {
        code: _.invalid_type,
        expected: S.array,
        received: n.parsedType
      }), T;
    if (n.data.length < this._def.items.length)
      return w(n, {
        code: _.too_small,
        minimum: this._def.items.length,
        inclusive: !0,
        exact: !1,
        type: "array"
      }), T;
    !this._def.rest && n.data.length > this._def.items.length && (w(n, {
      code: _.too_big,
      maximum: this._def.items.length,
      inclusive: !0,
      exact: !1,
      type: "array"
    }), r.dirty());
    const i = [...n.data].map((a, l) => {
      const c = this._def.items[l] || this._def.rest;
      return c ? c._parse(new We(n, a, n.path, l)) : null;
    }).filter((a) => !!a);
    return n.common.async ? Promise.all(i).then((a) => de.mergeArray(r, a)) : de.mergeArray(r, i);
  }
  get items() {
    return this._def.items;
  }
  rest(e) {
    return new Tt({
      ...this._def,
      rest: e
    });
  }
}
Tt.create = (t, e) => {
  if (!Array.isArray(t))
    throw new Error("You must pass an array of schemas to z.tuple([ ... ])");
  return new Tt({
    items: t,
    typeName: A.ZodTuple,
    rest: null,
    ...P(e)
  });
};
class on extends N {
  get keySchema() {
    return this._def.keyType;
  }
  get valueSchema() {
    return this._def.valueType;
  }
  _parse(e) {
    const { status: r, ctx: n } = this._processInputParams(e);
    if (n.parsedType !== S.object)
      return w(n, {
        code: _.invalid_type,
        expected: S.object,
        received: n.parsedType
      }), T;
    const s = [], i = this._def.keyType, a = this._def.valueType;
    for (const l in n.data)
      s.push({
        key: i._parse(new We(n, l, n.path, l)),
        value: a._parse(new We(n, n.data[l], n.path, l)),
        alwaysSet: l in n.data
      });
    return n.common.async ? de.mergeObjectAsync(r, s) : de.mergeObjectSync(r, s);
  }
  get element() {
    return this._def.valueType;
  }
  static create(e, r, n) {
    return r instanceof N ? new on({
      keyType: e,
      valueType: r,
      typeName: A.ZodRecord,
      ...P(n)
    }) : new on({
      keyType: Fe.create(),
      valueType: e,
      typeName: A.ZodRecord,
      ...P(r)
    });
  }
}
class pi extends N {
  get keySchema() {
    return this._def.keyType;
  }
  get valueSchema() {
    return this._def.valueType;
  }
  _parse(e) {
    const { status: r, ctx: n } = this._processInputParams(e);
    if (n.parsedType !== S.map)
      return w(n, {
        code: _.invalid_type,
        expected: S.map,
        received: n.parsedType
      }), T;
    const s = this._def.keyType, i = this._def.valueType, a = [...n.data.entries()].map(([l, c], f) => ({
      key: s._parse(new We(n, l, n.path, [f, "key"])),
      value: i._parse(new We(n, c, n.path, [f, "value"]))
    }));
    if (n.common.async) {
      const l = /* @__PURE__ */ new Map();
      return Promise.resolve().then(async () => {
        for (const c of a) {
          const f = await c.key, d = await c.value;
          if (f.status === "aborted" || d.status === "aborted")
            return T;
          (f.status === "dirty" || d.status === "dirty") && r.dirty(), l.set(f.value, d.value);
        }
        return { status: r.value, value: l };
      });
    } else {
      const l = /* @__PURE__ */ new Map();
      for (const c of a) {
        const f = c.key, d = c.value;
        if (f.status === "aborted" || d.status === "aborted")
          return T;
        (f.status === "dirty" || d.status === "dirty") && r.dirty(), l.set(f.value, d.value);
      }
      return { status: r.value, value: l };
    }
  }
}
pi.create = (t, e, r) => new pi({
  valueType: e,
  keyType: t,
  typeName: A.ZodMap,
  ...P(r)
});
class Pr extends N {
  _parse(e) {
    const { status: r, ctx: n } = this._processInputParams(e);
    if (n.parsedType !== S.set)
      return w(n, {
        code: _.invalid_type,
        expected: S.set,
        received: n.parsedType
      }), T;
    const s = this._def;
    s.minSize !== null && n.data.size < s.minSize.value && (w(n, {
      code: _.too_small,
      minimum: s.minSize.value,
      type: "set",
      inclusive: !0,
      exact: !1,
      message: s.minSize.message
    }), r.dirty()), s.maxSize !== null && n.data.size > s.maxSize.value && (w(n, {
      code: _.too_big,
      maximum: s.maxSize.value,
      type: "set",
      inclusive: !0,
      exact: !1,
      message: s.maxSize.message
    }), r.dirty());
    const i = this._def.valueType;
    function a(c) {
      const f = /* @__PURE__ */ new Set();
      for (const d of c) {
        if (d.status === "aborted")
          return T;
        d.status === "dirty" && r.dirty(), f.add(d.value);
      }
      return { status: r.value, value: f };
    }
    const l = [...n.data.values()].map((c, f) => i._parse(new We(n, c, n.path, f)));
    return n.common.async ? Promise.all(l).then((c) => a(c)) : a(l);
  }
  min(e, r) {
    return new Pr({
      ...this._def,
      minSize: { value: e, message: x.toString(r) }
    });
  }
  max(e, r) {
    return new Pr({
      ...this._def,
      maxSize: { value: e, message: x.toString(r) }
    });
  }
  size(e, r) {
    return this.min(e, r).max(e, r);
  }
  nonempty(e) {
    return this.min(1, e);
  }
}
Pr.create = (t, e) => new Pr({
  valueType: t,
  minSize: null,
  maxSize: null,
  typeName: A.ZodSet,
  ...P(e)
});
class mi extends N {
  get schema() {
    return this._def.getter();
  }
  _parse(e) {
    const { ctx: r } = this._processInputParams(e);
    return this._def.getter()._parse({ data: r.data, path: r.path, parent: r });
  }
}
mi.create = (t, e) => new mi({
  getter: t,
  typeName: A.ZodLazy,
  ...P(e)
});
class ls extends N {
  _parse(e) {
    if (e.data !== this._def.value) {
      const r = this._getOrReturnCtx(e);
      return w(r, {
        received: r.data,
        code: _.invalid_literal,
        expected: this._def.value
      }), T;
    }
    return { status: "valid", value: e.data };
  }
  get value() {
    return this._def.value;
  }
}
ls.create = (t, e) => new ls({
  value: t,
  typeName: A.ZodLiteral,
  ...P(e)
});
function Va(t, e) {
  return new rr({
    values: t,
    typeName: A.ZodEnum,
    ...P(e)
  });
}
class rr extends N {
  _parse(e) {
    if (typeof e.data != "string") {
      const r = this._getOrReturnCtx(e), n = this._def.values;
      return w(r, {
        expected: F.joinValues(n),
        received: r.parsedType,
        code: _.invalid_type
      }), T;
    }
    if (this._cache || (this._cache = new Set(this._def.values)), !this._cache.has(e.data)) {
      const r = this._getOrReturnCtx(e), n = this._def.values;
      return w(r, {
        received: r.data,
        code: _.invalid_enum_value,
        options: n
      }), T;
    }
    return xe(e.data);
  }
  get options() {
    return this._def.values;
  }
  get enum() {
    const e = {};
    for (const r of this._def.values)
      e[r] = r;
    return e;
  }
  get Values() {
    const e = {};
    for (const r of this._def.values)
      e[r] = r;
    return e;
  }
  get Enum() {
    const e = {};
    for (const r of this._def.values)
      e[r] = r;
    return e;
  }
  extract(e, r = this._def) {
    return rr.create(e, {
      ...this._def,
      ...r
    });
  }
  exclude(e, r = this._def) {
    return rr.create(this.options.filter((n) => !e.includes(n)), {
      ...this._def,
      ...r
    });
  }
}
rr.create = Va;
class gi extends N {
  _parse(e) {
    const r = F.getValidEnumValues(this._def.values), n = this._getOrReturnCtx(e);
    if (n.parsedType !== S.string && n.parsedType !== S.number) {
      const s = F.objectValues(r);
      return w(n, {
        expected: F.joinValues(s),
        received: n.parsedType,
        code: _.invalid_type
      }), T;
    }
    if (this._cache || (this._cache = new Set(F.getValidEnumValues(this._def.values))), !this._cache.has(e.data)) {
      const s = F.objectValues(r);
      return w(n, {
        received: n.data,
        code: _.invalid_enum_value,
        options: s
      }), T;
    }
    return xe(e.data);
  }
  get enum() {
    return this._def.values;
  }
}
gi.create = (t, e) => new gi({
  values: t,
  typeName: A.ZodNativeEnum,
  ...P(e)
});
class ln extends N {
  unwrap() {
    return this._def.type;
  }
  _parse(e) {
    const { ctx: r } = this._processInputParams(e);
    if (r.parsedType !== S.promise && r.common.async === !1)
      return w(r, {
        code: _.invalid_type,
        expected: S.promise,
        received: r.parsedType
      }), T;
    const n = r.parsedType === S.promise ? r.data : Promise.resolve(r.data);
    return xe(n.then((s) => this._def.type.parseAsync(s, {
      path: r.path,
      errorMap: r.common.contextualErrorMap
    })));
  }
}
ln.create = (t, e) => new ln({
  type: t,
  typeName: A.ZodPromise,
  ...P(e)
});
class Et extends N {
  innerType() {
    return this._def.schema;
  }
  sourceType() {
    return this._def.schema._def.typeName === A.ZodEffects ? this._def.schema.sourceType() : this._def.schema;
  }
  _parse(e) {
    const { status: r, ctx: n } = this._processInputParams(e), s = this._def.effect || null, i = {
      addIssue: (a) => {
        w(n, a), a.fatal ? r.abort() : r.dirty();
      },
      get path() {
        return n.path;
      }
    };
    if (i.addIssue = i.addIssue.bind(i), s.type === "preprocess") {
      const a = s.transform(n.data, i);
      if (n.common.async)
        return Promise.resolve(a).then(async (l) => {
          if (r.value === "aborted")
            return T;
          const c = await this._def.schema._parseAsync({
            data: l,
            path: n.path,
            parent: n
          });
          return c.status === "aborted" ? T : c.status === "dirty" || r.value === "dirty" ? Ar(c.value) : c;
        });
      {
        if (r.value === "aborted")
          return T;
        const l = this._def.schema._parseSync({
          data: a,
          path: n.path,
          parent: n
        });
        return l.status === "aborted" ? T : l.status === "dirty" || r.value === "dirty" ? Ar(l.value) : l;
      }
    }
    if (s.type === "refinement") {
      const a = (l) => {
        const c = s.refinement(l, i);
        if (n.common.async)
          return Promise.resolve(c);
        if (c instanceof Promise)
          throw new Error("Async refinement encountered during synchronous parse operation. Use .parseAsync instead.");
        return l;
      };
      if (n.common.async === !1) {
        const l = this._def.schema._parseSync({
          data: n.data,
          path: n.path,
          parent: n
        });
        return l.status === "aborted" ? T : (l.status === "dirty" && r.dirty(), a(l.value), { status: r.value, value: l.value });
      } else
        return this._def.schema._parseAsync({ data: n.data, path: n.path, parent: n }).then((l) => l.status === "aborted" ? T : (l.status === "dirty" && r.dirty(), a(l.value).then(() => ({ status: r.value, value: l.value }))));
    }
    if (s.type === "transform")
      if (n.common.async === !1) {
        const a = this._def.schema._parseSync({
          data: n.data,
          path: n.path,
          parent: n
        });
        if (!er(a))
          return T;
        const l = s.transform(a.value, i);
        if (l instanceof Promise)
          throw new Error("Asynchronous transform encountered during synchronous parse operation. Use .parseAsync instead.");
        return { status: r.value, value: l };
      } else
        return this._def.schema._parseAsync({ data: n.data, path: n.path, parent: n }).then((a) => er(a) ? Promise.resolve(s.transform(a.value, i)).then((l) => ({
          status: r.value,
          value: l
        })) : T);
    F.assertNever(s);
  }
}
Et.create = (t, e, r) => new Et({
  schema: t,
  typeName: A.ZodEffects,
  effect: e,
  ...P(r)
});
Et.createWithPreprocess = (t, e, r) => new Et({
  schema: e,
  effect: { type: "preprocess", transform: t },
  typeName: A.ZodEffects,
  ...P(r)
});
class lt extends N {
  _parse(e) {
    return this._getType(e) === S.undefined ? xe(void 0) : this._def.innerType._parse(e);
  }
  unwrap() {
    return this._def.innerType;
  }
}
lt.create = (t, e) => new lt({
  innerType: t,
  typeName: A.ZodOptional,
  ...P(e)
});
class nr extends N {
  _parse(e) {
    return this._getType(e) === S.null ? xe(null) : this._def.innerType._parse(e);
  }
  unwrap() {
    return this._def.innerType;
  }
}
nr.create = (t, e) => new nr({
  innerType: t,
  typeName: A.ZodNullable,
  ...P(e)
});
class cs extends N {
  _parse(e) {
    const { ctx: r } = this._processInputParams(e);
    let n = r.data;
    return r.parsedType === S.undefined && (n = this._def.defaultValue()), this._def.innerType._parse({
      data: n,
      path: r.path,
      parent: r
    });
  }
  removeDefault() {
    return this._def.innerType;
  }
}
cs.create = (t, e) => new cs({
  innerType: t,
  typeName: A.ZodDefault,
  defaultValue: typeof e.default == "function" ? e.default : () => e.default,
  ...P(e)
});
class us extends N {
  _parse(e) {
    const { ctx: r } = this._processInputParams(e), n = {
      ...r,
      common: {
        ...r.common,
        issues: []
      }
    }, s = this._def.innerType._parse({
      data: n.data,
      path: n.path,
      parent: {
        ...n
      }
    });
    return rn(s) ? s.then((i) => ({
      status: "valid",
      value: i.status === "valid" ? i.value : this._def.catchValue({
        get error() {
          return new Re(n.common.issues);
        },
        input: n.data
      })
    })) : {
      status: "valid",
      value: s.status === "valid" ? s.value : this._def.catchValue({
        get error() {
          return new Re(n.common.issues);
        },
        input: n.data
      })
    };
  }
  removeCatch() {
    return this._def.innerType;
  }
}
us.create = (t, e) => new us({
  innerType: t,
  typeName: A.ZodCatch,
  catchValue: typeof e.catch == "function" ? e.catch : () => e.catch,
  ...P(e)
});
class vi extends N {
  _parse(e) {
    if (this._getType(e) !== S.nan) {
      const n = this._getOrReturnCtx(e);
      return w(n, {
        code: _.invalid_type,
        expected: S.nan,
        received: n.parsedType
      }), T;
    }
    return { status: "valid", value: e.data };
  }
}
vi.create = (t) => new vi({
  typeName: A.ZodNaN,
  ...P(t)
});
class Ru extends N {
  _parse(e) {
    const { ctx: r } = this._processInputParams(e), n = r.data;
    return this._def.type._parse({
      data: n,
      path: r.path,
      parent: r
    });
  }
  unwrap() {
    return this._def.type;
  }
}
class Ss extends N {
  _parse(e) {
    const { status: r, ctx: n } = this._processInputParams(e);
    if (n.common.async)
      return (async () => {
        const i = await this._def.in._parseAsync({
          data: n.data,
          path: n.path,
          parent: n
        });
        return i.status === "aborted" ? T : i.status === "dirty" ? (r.dirty(), Ar(i.value)) : this._def.out._parseAsync({
          data: i.value,
          path: n.path,
          parent: n
        });
      })();
    {
      const s = this._def.in._parseSync({
        data: n.data,
        path: n.path,
        parent: n
      });
      return s.status === "aborted" ? T : s.status === "dirty" ? (r.dirty(), {
        status: "dirty",
        value: s.value
      }) : this._def.out._parseSync({
        data: s.value,
        path: n.path,
        parent: n
      });
    }
  }
  static create(e, r) {
    return new Ss({
      in: e,
      out: r,
      typeName: A.ZodPipeline
    });
  }
}
class ds extends N {
  _parse(e) {
    const r = this._def.innerType._parse(e), n = (s) => (er(s) && (s.value = Object.freeze(s.value)), s);
    return rn(r) ? r.then((s) => n(s)) : n(r);
  }
  unwrap() {
    return this._def.innerType;
  }
}
ds.create = (t, e) => new ds({
  innerType: t,
  typeName: A.ZodReadonly,
  ...P(e)
});
var A;
(function(t) {
  t.ZodString = "ZodString", t.ZodNumber = "ZodNumber", t.ZodNaN = "ZodNaN", t.ZodBigInt = "ZodBigInt", t.ZodBoolean = "ZodBoolean", t.ZodDate = "ZodDate", t.ZodSymbol = "ZodSymbol", t.ZodUndefined = "ZodUndefined", t.ZodNull = "ZodNull", t.ZodAny = "ZodAny", t.ZodUnknown = "ZodUnknown", t.ZodNever = "ZodNever", t.ZodVoid = "ZodVoid", t.ZodArray = "ZodArray", t.ZodObject = "ZodObject", t.ZodUnion = "ZodUnion", t.ZodDiscriminatedUnion = "ZodDiscriminatedUnion", t.ZodIntersection = "ZodIntersection", t.ZodTuple = "ZodTuple", t.ZodRecord = "ZodRecord", t.ZodMap = "ZodMap", t.ZodSet = "ZodSet", t.ZodFunction = "ZodFunction", t.ZodLazy = "ZodLazy", t.ZodLiteral = "ZodLiteral", t.ZodEnum = "ZodEnum", t.ZodEffects = "ZodEffects", t.ZodNativeEnum = "ZodNativeEnum", t.ZodOptional = "ZodOptional", t.ZodNullable = "ZodNullable", t.ZodDefault = "ZodDefault", t.ZodCatch = "ZodCatch", t.ZodPromise = "ZodPromise", t.ZodBranded = "ZodBranded", t.ZodPipeline = "ZodPipeline", t.ZodReadonly = "ZodReadonly";
})(A || (A = {}));
const O = Fe.create, sr = xt.create;
Ct.create;
const et = nn.create;
tr.create;
const be = as.create;
dt.create;
const tt = ze.create, k = re.create, Se = sn.create;
an.create;
Tt.create;
const _n = on.create, xs = ls.create, le = rr.create;
ln.create;
lt.create;
nr.create;
const K = Et.createWithPreprocess, Z = {
  string: ((t) => Fe.create({ ...t, coerce: !0 })),
  number: ((t) => xt.create({ ...t, coerce: !0 })),
  boolean: ((t) => nn.create({
    ...t,
    coerce: !0
  })),
  bigint: ((t) => Ct.create({ ...t, coerce: !0 })),
  date: ((t) => tr.create({ ...t, coerce: !0 }))
};
class An extends Error {
  constructor(e, r = "UNKNOWN_ERROR") {
    super(e), this.name = this.constructor.name, this.code = r, Error.captureStackTrace && Error.captureStackTrace(this, this.constructor);
  }
}
class In extends An {
  constructor(e, r) {
    super(e, "VALIDATION_ERROR"), this.details = r;
  }
}
class hr extends An {
  constructor(e, r) {
    super(e, "DATA_ERROR"), this.path = r;
  }
}
class Me extends An {
  constructor(e, r, n) {
    super(e, "EXPRESSION_ERROR"), this.expression = r, this.details = n;
  }
}
class Wt extends An {
  constructor(e) {
    super(e, "STATE_ERROR");
  }
}
function fs(t) {
  return t && typeof t == "object" && "value" in t && "peek" in t;
}
function B(t, e) {
  return {
    name: t.name,
    returnType: t.returnType,
    schema: t.schema,
    execute: e
  };
}
class Cs {
  constructor(e, r, n = [], s) {
    this.id = e;
    const i = /* @__PURE__ */ new Map();
    for (const l of r)
      i.set(l.name, l);
    this.components = i;
    const a = /* @__PURE__ */ new Map();
    for (const l of n)
      a.set(l.name, l);
    this.functions = a, this.themeSchema = s, this.invoker = (l, c, f, d) => {
      const u = this.functions.get(l);
      if (!u)
        throw new Me(`Function not found in catalog '${this.id}': ${l}`, l);
      try {
        const o = u.schema.parse(c);
        return u.execute(o, f, d);
      } catch (o) {
        throw o?.name === "ZodError" || o instanceof Re ? new Me(`Validation failed for function '${l}': ${o.message}`, l, o.errors ?? o.issues) : o;
      }
    };
  }
}
class ct {
  constructor() {
    this.listeners = /* @__PURE__ */ new Set();
  }
  /**
   * Subscribes to the event.
   *
   * @param listener The listener function to call when the event is emitted.
   * @returns A subscription object that can be used to unsubscribe.
   */
  subscribe(e) {
    return this.listeners.add(e), {
      unsubscribe: () => this.listeners.delete(e)
    };
  }
  /**
   * Emits an event to all subscribers.
   *
   * @param data The data to pass to subscribers.
   */
  async emit(e) {
    for (const r of this.listeners)
      try {
        await r(e);
      } catch (n) {
        console.error("EventEmitter error:", n);
      }
  }
  /**
   * Removes all listeners.
   */
  dispose() {
    this.listeners.clear();
  }
}
var Lu = Symbol.for("preact-signals");
function wn() {
  if (Je > 1)
    Je--;
  else {
    var t, e = !1;
    for ((function() {
      var s = un;
      for (un = void 0; s !== void 0; ) {
        var i = s.S;
        if (i.v === s.v) for (var a = i.t; a !== void 0; a = a.x) a.i === s.i && (a.i = i.i);
        s = s.o;
      }
    })(); Cr !== void 0; ) {
      var r = Cr;
      for (Cr = void 0, cn++; r !== void 0; ) {
        var n = r.u;
        if (r.u = void 0, r.f &= -3, !(8 & r.f) && Ba(r)) try {
          r.c();
        } catch (s) {
          e || (t = s, e = !0);
        }
        r = n;
      }
    }
    if (cn = 0, Je--, e) throw t;
  }
}
function bi(t) {
  if (Je > 0) return t();
  hs = ++Mu, Je++;
  try {
    return t();
  } finally {
    wn();
  }
}
var xr, q = void 0;
function kn(t) {
  var e = q, r = xr;
  q = void 0, xr = void 0;
  try {
    return t();
  } finally {
    q = e, xr = r;
  }
}
var Cr = void 0, Je = 0, cn = 0, Mu = 0, hs = 0, un = void 0, dn = 0;
function Wa(t) {
  if (q !== void 0) {
    var e = t.n;
    if (e === void 0 || e.t !== q)
      return e = { i: 0, S: t, p: q.s, n: void 0, t: q, e: void 0, x: void 0, r: e }, q.s !== void 0 && (q.s.n = e), q.s = e, t.n = e, 32 & q.f && t.S(e), e;
    if (e.i === -1)
      return e.i = 0, e.n !== void 0 && (e.n.p = e.p, e.p !== void 0 && (e.p.n = e.n), e.p = q.s, e.n = void 0, q.s.n = e, q.s = e), e;
  }
}
function ce(t, e) {
  this.v = t, this.i = 0, this.n = void 0, this.t = void 0, this.l = 0, this.W = e?.watched, this.Z = e?.unwatched, this.name = e?.name;
}
ce.prototype.brand = Lu;
ce.prototype.h = function() {
  return !0;
};
ce.prototype.S = function(t) {
  var e = this, r = this.t;
  r !== t && t.e === void 0 && (t.x = r, this.t = t, r !== void 0 ? r.e = t : kn(function() {
    var n;
    (n = e.W) == null || n.call(e);
  }));
};
ce.prototype.U = function(t) {
  var e = this;
  if (this.t !== void 0) {
    var r = t.e, n = t.x;
    r !== void 0 && (r.x = n, t.e = void 0), n !== void 0 && (n.e = r, t.x = void 0), t === this.t && (this.t = n, n === void 0 && kn(function() {
      var s;
      (s = e.Z) == null || s.call(e);
    }));
  }
};
ce.prototype.subscribe = function(t) {
  var e = this;
  return Tr(function() {
    var r = e.value;
    kn(function() {
      return t(r);
    });
  }, { name: "sub" });
};
ce.prototype.valueOf = function() {
  return this.value;
};
ce.prototype.toString = function() {
  return this.value + "";
};
ce.prototype.toJSON = function() {
  return this.value;
};
ce.prototype.peek = function() {
  var t = this;
  return kn(function() {
    return t.value;
  });
};
Object.defineProperty(ce.prototype, "value", { get: function() {
  var t = Wa(this);
  return t !== void 0 && (t.i = this.i), this.v;
}, set: function(t) {
  if (t !== this.v) {
    if (cn > 100) throw new Error("Cycle detected");
    (function(r) {
      Je !== 0 && cn === 0 && r.l !== hs && (r.l = hs, un = { S: r, v: r.v, i: r.i, o: un });
    })(this), this.v = t, this.i++, dn++, Je++;
    try {
      for (var e = this.t; e !== void 0; e = e.x) e.t.N();
    } finally {
      wn();
    }
  }
} });
function wr(t, e) {
  return new ce(t, e);
}
function Ba(t) {
  for (var e = t.s; e !== void 0; e = e.n) if (e.S.i !== e.i || !e.S.h() || e.S.i !== e.i) return !0;
  return !1;
}
function qa(t) {
  for (var e = t.s; e !== void 0; e = e.n) {
    var r = e.S.n;
    if (r !== void 0 && (e.r = r), e.S.n = e, e.i = -1, e.n === void 0) {
      t.s = e;
      break;
    }
  }
}
function Ha(t) {
  for (var e = t.s, r = void 0; e !== void 0; ) {
    var n = e.p;
    e.i === -1 ? (e.S.U(e), n !== void 0 && (n.n = e.n), e.n !== void 0 && (e.n.p = n)) : r = e, e.S.n = e.r, e.r !== void 0 && (e.r = void 0), e = n;
  }
  t.s = r;
}
function Nt(t, e) {
  ce.call(this, void 0, e), this.x = t, this.s = void 0, this.g = dn - 1, this.f = 4;
}
Nt.prototype = new ce();
Nt.prototype.h = function() {
  if (this.f &= -3, 1 & this.f) return !1;
  if ((36 & this.f) == 32 || (this.f &= -5, this.g === dn)) return !0;
  if (this.g = dn, this.f |= 1, this.i > 0 && !Ba(this))
    return this.f &= -2, !0;
  var t = q;
  try {
    qa(this), q = this;
    var e = this.x();
    (16 & this.f || this.v !== e || this.i === 0) && (this.v = e, this.f &= -17, this.i++);
  } catch (r) {
    this.v = r, this.f |= 16, this.i++;
  }
  return q = t, Ha(this), this.f &= -2, !0;
};
Nt.prototype.S = function(t) {
  if (this.t === void 0) {
    this.f |= 36;
    for (var e = this.s; e !== void 0; e = e.n) e.S.S(e);
  }
  ce.prototype.S.call(this, t);
};
Nt.prototype.U = function(t) {
  if (this.t !== void 0 && (ce.prototype.U.call(this, t), this.t === void 0)) {
    this.f &= -33;
    for (var e = this.s; e !== void 0; e = e.n) e.S.U(e);
  }
};
Nt.prototype.N = function() {
  if (!(2 & this.f)) {
    this.f |= 6;
    for (var t = this.t; t !== void 0; t = t.x) t.t.N();
  }
};
Object.defineProperty(Nt.prototype, "value", { get: function() {
  if (1 & this.f) throw new Error("Cycle detected");
  var t = Wa(this);
  if (this.h(), t !== void 0 && (t.i = this.i), 16 & this.f) throw this.v;
  return this.v;
} });
function Ya(t, e) {
  return new Nt(t, e);
}
function Ga(t) {
  var e = t.m;
  if (t.m = void 0, typeof e == "function") {
    Je++;
    var r = q;
    q = void 0;
    try {
      e();
    } catch (n) {
      throw t.f &= -2, t.f |= 8, Ts(t), n;
    } finally {
      q = r, wn();
    }
  }
}
function Ts(t) {
  for (var e = t.s; e !== void 0; e = e.n) e.S.U(e);
  t.x = void 0, t.s = void 0, Ga(t);
}
function Fu(t) {
  if (q !== this) throw new Error("Out-of-order effect");
  Ha(this), q = t, this.f &= -2, 8 & this.f && Ts(this), wn();
}
function lr(t, e) {
  this.x = t, this.m = void 0, this.s = void 0, this.u = void 0, this.f = 32, this.name = e?.name, xr && xr.push(this);
}
lr.prototype.c = function() {
  var t = this.S();
  try {
    if (8 & this.f || this.x === void 0) return;
    var e = this.x();
    typeof e == "function" && (this.m = e);
  } finally {
    t();
  }
};
lr.prototype.S = function() {
  if (1 & this.f) throw new Error("Cycle detected");
  this.f |= 1, this.f &= -9, Ga(this), qa(this), Je++;
  var t = q;
  return q = this, Fu.bind(this, t);
};
lr.prototype.N = function() {
  2 & this.f || (this.f |= 2, this.u = Cr, Cr = this);
};
lr.prototype.d = function() {
  this.f |= 8, 1 & this.f || Ts(this);
};
lr.prototype.dispose = function() {
  this.d();
};
function Tr(t, e) {
  var r = new lr(t, e);
  try {
    r.c();
  } catch (s) {
    throw r.d(), s;
  }
  var n = r.d.bind(r);
  return n[Symbol.dispose] = n, n;
}
function zn(t) {
  return /^\d+$/.test(t);
}
class Iu {
  /**
   * Creates a new data model.
   *
   * @param initialData The initial data for the model. Defaults to an empty object.
   */
  constructor(e = {}) {
    this.data = {}, this.signals = /* @__PURE__ */ new Map(), this.subscriptions = /* @__PURE__ */ new Set(), this.data = e;
  }
  /**
   * Retrieves a Preact Signal for a specific data path.
   *
   * This provides a reactive way to access a value. If the value at the path changes via `set()`,
   * the signal will automatically be updated.
   *
   * @param path The JSON pointer path to create or retrieve a signal for.
   * @returns A Preact Signal representing the value at the specified path.
   */
  getSignal(e) {
    const r = this.normalizePath(e);
    return this.signals.has(r) || this.signals.set(r, wr(this.get(r))), this.signals.get(r);
  }
  /**
   * Updates the model at the specific path and notifies all relevant signals.
   * If path is '/' or empty, replaces the entire root.
   *
   * Note on `undefined` values:
   * - For objects: Setting a property to `undefined` removes the key from the object.
   * - For arrays: Setting an index to `undefined` sets that index to `undefined` but preserves the array length (sparse array).
   */
  set(e, r) {
    if (e == null)
      throw new hr("Path cannot be null or undefined.");
    if (e === "/" || e === "")
      return this.data = r, this.notifyAllSignals(), this;
    const n = this.parsePath(e), s = n.pop();
    this.data || (this.data = {});
    let i = this.data;
    for (let a = 0; a < n.length; a++) {
      const l = n[a];
      if (Array.isArray(i) && !zn(l))
        throw new hr(`Cannot use non-numeric segment '${l}' on an array in path '${e}'.`, e);
      if (i[l] !== void 0 && i[l] !== null && typeof i[l] != "object")
        throw new hr(`Cannot set path '${e}': segment '${l}' is a primitive value.`, e);
      if (i[l] === void 0 || i[l] === null) {
        const c = a < n.length - 1 ? n[a + 1] : s;
        i[l] = zn(c) ? [] : {};
      }
      i = i[l];
    }
    if (Array.isArray(i) && !zn(s))
      throw new hr(`Cannot use non-numeric segment '${s}' on an array in path '${e}'.`, e);
    return r === void 0 ? Array.isArray(i) ? i[parseInt(s, 10)] = void 0 : delete i[s] : i[s] = r, this.notifySignals(e), this;
  }
  /**
   * Retrieves data at a specific JSON pointer path.
   *
   * @param path The JSON pointer path to read from.
   * @returns The value at the specified path, or undefined if not found.
   */
  get(e) {
    if (e == null)
      throw new hr("Path cannot be null or undefined.");
    if (e === "/" || e === "")
      return this.data;
    const r = this.parsePath(e);
    let n = this.data;
    for (const s of r) {
      if (n == null)
        return;
      n = n[s];
    }
    return n;
  }
  /**
   * Subscribes to changes at the specified data path.
   *
   * This is a backwards-compatible layer using Preact Signals internally. It allows
   * listeners to be notified whenever the value at the specified path (or any of its
   * ancestors/descendants) changes.
   *
   * @param path The JSON pointer path to observe.
   * @param onChange A callback fired whenever the value changes.
   * @returns A `DataSubscription` containing the initial value and an `unsubscribe` method.
   */
  subscribe(e, r) {
    const n = this.getSignal(e);
    let s = !0, i = n.peek();
    const a = Tr(() => {
      const l = n.value;
      i = l, s || r(l);
    });
    return s = !1, this.subscriptions.add(a), {
      get value() {
        return i;
      },
      unsubscribe: () => {
        a(), this.subscriptions.delete(a);
      }
    };
  }
  /**
   * Clears all internal subscriptions.
   */
  dispose() {
    for (const e of this.subscriptions)
      e();
    this.subscriptions.clear(), this.signals.clear();
  }
  normalizePath(e) {
    return e.length > 1 && e.endsWith("/") ? e.slice(0, -1) : e || "/";
  }
  parsePath(e) {
    return e.split("/").filter((r) => r.length > 0);
  }
  notifySignals(e) {
    const r = this.normalizePath(e);
    bi(() => {
      this.updateSignal(r);
      let n = r;
      for (; n !== "/" && n !== ""; )
        n = n.substring(0, n.lastIndexOf("/")) || "/", this.updateSignal(n);
      for (const s of this.signals.keys())
        this.isDescendant(s, r) && this.updateSignal(s);
    });
  }
  updateSignal(e) {
    const r = this.signals.get(e);
    if (r) {
      const n = this.get(e);
      Array.isArray(n) ? r.value = [...n] : typeof n == "object" && n !== null ? r.value = { ...n } : r.value = n;
    }
  }
  notifyAllSignals() {
    bi(() => {
      for (const e of this.signals.keys())
        this.updateSignal(e);
    });
  }
  isDescendant(e, r) {
    return r === "/" || r === "" ? e !== "/" : e.startsWith(r + "/");
  }
}
class zu {
  constructor() {
    this.components = /* @__PURE__ */ new Map(), this._onCreated = new ct(), this._onDeleted = new ct(), this.onCreated = this._onCreated, this.onDeleted = this._onDeleted;
  }
  /**
   * Retrieves a component by its ID.
   *
   *
   * @param id The ID of the component to retrieve.
   * @returns The component model, or undefined if not found.
   */
  get(e) {
    return this.components.get(e);
  }
  /**
   * Returns an iterator over the components in the model.
   */
  get entries() {
    return this.components.entries();
  }
  /**
   * Adds a component to the model.
   * Throws an error if a component with the same ID already exists.
   *
   * @param component The component to add.
   */
  addComponent(e) {
    if (this.components.has(e.id))
      throw new Wt(`Component with id '${e.id}' already exists.`);
    this.components.set(e.id, e), this._onCreated.emit(e);
  }
  /**
   * Removes a component from the model by its ID.
   * Disposes of the component upon removal.
   *
   * @param id The ID of the component to remove.
   */
  removeComponent(e) {
    const r = this.components.get(e);
    r && (this.components.delete(e), r.dispose(), this._onDeleted.emit(e));
  }
  /**
   * Disposes of the model and all its components.
   */
  dispose() {
    for (const e of this.components.values())
      e.dispose();
    this.components.clear(), this._onCreated.dispose(), this._onDeleted.dispose();
  }
}
const Ja = k({
  name: O().describe("The name of the action, taken from the component's action.event.name property."),
  surfaceId: O().describe("The id of the surface where the event originated."),
  sourceComponentId: O().describe("The id of the component that triggered the event."),
  timestamp: O().datetime().describe("An ISO 8601 timestamp of when the event occurred."),
  context: _n(be()).describe("A JSON object containing the key-value pairs from the component's action.event.context, after resolving all data bindings.")
}).strict(), Zu = k({
  code: xs("VALIDATION_FAILED"),
  surfaceId: O().describe("The id of the surface where the error occurred."),
  path: O().describe("The JSON pointer to the field that failed validation (e.g. '/components/0/text')."),
  message: O().describe("A short one or two sentence description of why validation failed.")
}).strict(), Uu = k({
  code: O().refine((t) => t !== "VALIDATION_FAILED"),
  message: O().describe("A short one or two sentence description of why the error occurred."),
  surfaceId: O().describe("The id of the surface where the error occurred.")
}).passthrough(), Vu = Se([
  Zu,
  Uu
]), Wu = k({
  version: xs("v0.9")
}).and(Se([
  k({ action: Ja }),
  k({ error: Vu })
]));
k({
  version: xs("v0.9"),
  surfaces: _n(k({}).passthrough()).describe("A map of surface IDs to their current data models.")
}).strict();
const Bu = tt(Wu).describe("A list of client messages.");
k({
  messages: Bu
}).strict().describe("An object wrapping a list of client messages.");
class qu {
  /**
   * Creates a new surface model.
   *
   * @param id The unique identifier for this surface.
   * @param catalog The component catalog used by this surface.
   * @param theme The theme to apply to this surface.
   * @param sendDataModel If true, the client will send the full data model.
   */
  constructor(e, r, n = {}, s = !1) {
    this.id = e, this.catalog = r, this.theme = n, this.sendDataModel = s, this._onAction = new ct(), this._onError = new ct(), this.onAction = this._onAction, this.onError = this._onError, this.dataModel = new Iu({}), this.componentsModel = new zu();
  }
  /**
   * Dispatches an action from this surface to listeners.
   *
   * @param payload The action payload (name and context) to dispatch.
   * @param sourceComponentId The ID of the component that triggered the action.
   */
  async dispatchAction(e, r) {
    if (e && typeof e == "object" && "event" in e && e.event) {
      const n = {
        name: e.event.name,
        surfaceId: this.id,
        sourceComponentId: r,
        timestamp: (/* @__PURE__ */ new Date()).toISOString(),
        context: e.event.context || {}
      }, s = Ja.safeParse(n);
      s.success ? await this._onAction.emit(s.data) : console.error("A2UI: Invalid action payload dispatched.", s.error.format());
    }
  }
  /**
   * Dispatches an error from this surface to listeners.
   *
   * @param error The error object to dispatch, conforming to client_to_server schema.
   */
  async dispatchError(e) {
    await this._onError.emit({
      ...e,
      surfaceId: this.id
    });
  }
  /**
   * Disposes of the surface and its resources.
   */
  dispose() {
    this.dataModel.dispose(), this.componentsModel.dispose(), this._onAction.dispose(), this._onError.dispose();
  }
}
class Hu {
  constructor() {
    this.surfaces = /* @__PURE__ */ new Map(), this.surfaceUnsubscribers = /* @__PURE__ */ new Map(), this._onSurfaceCreated = new ct(), this._onSurfaceDeleted = new ct(), this._onAction = new ct(), this.onSurfaceCreated = this._onSurfaceCreated, this.onSurfaceDeleted = this._onSurfaceDeleted, this.onAction = this._onAction;
  }
  /**
   * Adds a surface to the group.
   * Ignores if a surface with the same ID already exists.
   *
   * @param surface The surface model to add.
   */
  addSurface(e) {
    if (this.surfaces.has(e.id)) {
      console.warn(`Surface ${e.id} already exists. Ignoring.`);
      return;
    }
    this.surfaces.set(e.id, e);
    const r = e.onAction.subscribe((n) => this._onAction.emit(n));
    this.surfaceUnsubscribers.set(e.id, r), this._onSurfaceCreated.emit(e);
  }
  /**
   * Removes a surface from the group by its ID.
   * Disposes of the surface upon removal.
   *
   * @param id The ID of the surface to remove.
   */
  deleteSurface(e) {
    const r = this.surfaces.get(e);
    if (r) {
      const n = this.surfaceUnsubscribers.get(e);
      n && (n.unsubscribe(), this.surfaceUnsubscribers.delete(e)), this.surfaces.delete(e), r.dispose(), this._onSurfaceDeleted.emit(e);
    }
  }
  /**
   * Retrieves a surface by its ID.
   *
   *
   * @param id The ID of the surface to retrieve.
   * @returns The surface model, or undefined if not found.
   */
  getSurface(e) {
    return this.surfaces.get(e);
  }
  /**
   * Returns a readonly map of all active surfaces.
   */
  get surfacesMap() {
    return this.surfaces;
  }
  /**
   * Disposes of the group and all its surfaces.
   */
  dispose() {
    for (const e of Array.from(this.surfaces.keys()))
      this.deleteSurface(e);
    this._onSurfaceCreated.dispose(), this._onSurfaceDeleted.dispose(), this._onAction.dispose();
  }
}
class yi {
  /**
   * Creates a new component model.
   *
   * @param id The unique identifier for this component.
   * @param type The component type name.
   * @param initialProperties The initial properties for the component.
   */
  constructor(e, r, n) {
    this.id = e, this.type = r, this._onUpdated = new ct(), this.onUpdated = this._onUpdated, this._properties = n;
  }
  /**
   * The current properties of the component.
   */
  get properties() {
    return this._properties;
  }
  set properties(e) {
    this._properties = e, this._onUpdated.emit(this);
  }
  /**
   * Disposes of the component and its resources.
   */
  dispose() {
    this._onUpdated.dispose();
  }
  /**
   * Returns a JSON representation of the component tree.
   */
  get componentTree() {
    return {
      id: this.id,
      type: this.type,
      ...this._properties
    };
  }
}
const Yu = Symbol("Let zodToJsonSchema decide on which parser to use"), _i = {
  name: void 0,
  $refStrategy: "root",
  basePath: ["#"],
  effectStrategy: "input",
  pipeStrategy: "all",
  dateStrategy: "format:date-time",
  mapStrategy: "entries",
  removeAdditionalStrategy: "passthrough",
  allowedAdditionalProperties: !0,
  rejectedAdditionalProperties: !1,
  definitionPath: "definitions",
  target: "jsonSchema7",
  strictUnions: !1,
  definitions: {},
  errorMessages: !1,
  markdownDescription: !1,
  patternStrategy: "escape",
  applyRegexFlags: !1,
  emailStrategy: "format:email",
  base64Strategy: "contentEncoding:base64",
  nameStrategy: "ref",
  openAiAnyTypeName: "OpenAiAnyType"
}, Gu = (t) => typeof t == "string" ? {
  ..._i,
  name: t
} : {
  ..._i,
  ...t
}, Ju = (t) => {
  const e = Gu(t), r = e.name !== void 0 ? [...e.basePath, e.definitionPath, e.name] : e.basePath;
  return {
    ...e,
    flags: { hasReferencedOpenAiAnyType: !1 },
    currentPath: r,
    propertyPath: void 0,
    seen: new Map(Object.entries(e.definitions).map(([n, s]) => [
      s._def,
      {
        def: s._def,
        path: [...e.basePath, e.definitionPath, n],
        // Resolution of references will be forced even though seen, so it's ok that the schema is undefined here for now.
        jsonSchema: void 0
      }
    ]))
  };
};
function Xa(t, e, r, n) {
  n?.errorMessages && r && (t.errorMessage = {
    ...t.errorMessage,
    [e]: r
  });
}
function W(t, e, r, n, s) {
  t[e] = r, Xa(t, e, n, s);
}
const Qa = (t, e) => {
  let r = 0;
  for (; r < t.length && r < e.length && t[r] === e[r]; r++)
    ;
  return [(t.length - r).toString(), ...e.slice(r)].join("/");
};
function me(t) {
  if (t.target !== "openAi")
    return {};
  const e = [
    ...t.basePath,
    t.definitionPath,
    t.openAiAnyTypeName
  ];
  return t.flags.hasReferencedOpenAiAnyType = !0, {
    $ref: t.$refStrategy === "relative" ? Qa(e, t.currentPath) : e.join("/")
  };
}
function Xu(t, e) {
  const r = {
    type: "array"
  };
  return t.type?._def && t.type?._def?.typeName !== A.ZodAny && (r.items = U(t.type._def, {
    ...e,
    currentPath: [...e.currentPath, "items"]
  })), t.minLength && W(r, "minItems", t.minLength.value, t.minLength.message, e), t.maxLength && W(r, "maxItems", t.maxLength.value, t.maxLength.message, e), t.exactLength && (W(r, "minItems", t.exactLength.value, t.exactLength.message, e), W(r, "maxItems", t.exactLength.value, t.exactLength.message, e)), r;
}
function Qu(t, e) {
  const r = {
    type: "integer",
    format: "int64"
  };
  if (!t.checks)
    return r;
  for (const n of t.checks)
    switch (n.kind) {
      case "min":
        e.target === "jsonSchema7" ? n.inclusive ? W(r, "minimum", n.value, n.message, e) : W(r, "exclusiveMinimum", n.value, n.message, e) : (n.inclusive || (r.exclusiveMinimum = !0), W(r, "minimum", n.value, n.message, e));
        break;
      case "max":
        e.target === "jsonSchema7" ? n.inclusive ? W(r, "maximum", n.value, n.message, e) : W(r, "exclusiveMaximum", n.value, n.message, e) : (n.inclusive || (r.exclusiveMaximum = !0), W(r, "maximum", n.value, n.message, e));
        break;
      case "multipleOf":
        W(r, "multipleOf", n.value, n.message, e);
        break;
    }
  return r;
}
function Ku() {
  return {
    type: "boolean"
  };
}
function Ka(t, e) {
  return U(t.type._def, e);
}
const ed = (t, e) => U(t.innerType._def, e);
function eo(t, e, r) {
  const n = r ?? e.dateStrategy;
  if (Array.isArray(n))
    return {
      anyOf: n.map((s, i) => eo(t, e, s))
    };
  switch (n) {
    case "string":
    case "format:date-time":
      return {
        type: "string",
        format: "date-time"
      };
    case "format:date":
      return {
        type: "string",
        format: "date"
      };
    case "integer":
      return td(t, e);
  }
}
const td = (t, e) => {
  const r = {
    type: "integer",
    format: "unix-time"
  };
  if (e.target === "openApi3")
    return r;
  for (const n of t.checks)
    switch (n.kind) {
      case "min":
        W(
          r,
          "minimum",
          n.value,
          // This is in milliseconds
          n.message,
          e
        );
        break;
      case "max":
        W(
          r,
          "maximum",
          n.value,
          // This is in milliseconds
          n.message,
          e
        );
        break;
    }
  return r;
};
function rd(t, e) {
  return {
    ...U(t.innerType._def, e),
    default: t.defaultValue()
  };
}
function nd(t, e) {
  return e.effectStrategy === "input" ? U(t.schema._def, e) : me(e);
}
function sd(t) {
  return {
    type: "string",
    enum: Array.from(t.values)
  };
}
const id = (t) => "type" in t && t.type === "string" ? !1 : "allOf" in t;
function ad(t, e) {
  const r = [
    U(t.left._def, {
      ...e,
      currentPath: [...e.currentPath, "allOf", "0"]
    }),
    U(t.right._def, {
      ...e,
      currentPath: [...e.currentPath, "allOf", "1"]
    })
  ].filter((i) => !!i);
  let n = e.target === "jsonSchema2019-09" ? { unevaluatedProperties: !1 } : void 0;
  const s = [];
  return r.forEach((i) => {
    if (id(i))
      s.push(...i.allOf), i.unevaluatedProperties === void 0 && (n = void 0);
    else {
      let a = i;
      if ("additionalProperties" in i && i.additionalProperties === !1) {
        const { additionalProperties: l, ...c } = i;
        a = c;
      } else
        n = void 0;
      s.push(a);
    }
  }), s.length ? {
    allOf: s,
    ...n
  } : void 0;
}
function od(t, e) {
  const r = typeof t.value;
  return r !== "bigint" && r !== "number" && r !== "boolean" && r !== "string" ? {
    type: Array.isArray(t.value) ? "array" : "object"
  } : e.target === "openApi3" ? {
    type: r === "bigint" ? "integer" : r,
    enum: [t.value]
  } : {
    type: r === "bigint" ? "integer" : r,
    const: t.value
  };
}
let Zn;
const Oe = {
  /**
   * `c` was changed to `[cC]` to replicate /i flag
   */
  cuid: /^[cC][^\s-]{8,}$/,
  cuid2: /^[0-9a-z]+$/,
  ulid: /^[0-9A-HJKMNP-TV-Z]{26}$/,
  /**
   * `a-z` was added to replicate /i flag
   */
  email: /^(?!\.)(?!.*\.\.)([a-zA-Z0-9_'+\-\.]*)[a-zA-Z0-9_+-]@([a-zA-Z0-9][a-zA-Z0-9\-]*\.)+[a-zA-Z]{2,}$/,
  /**
   * Constructed a valid Unicode RegExp
   *
   * Lazily instantiate since this type of regex isn't supported
   * in all envs (e.g. React Native).
   *
   * See:
   * https://github.com/colinhacks/zod/issues/2433
   * Fix in Zod:
   * https://github.com/colinhacks/zod/commit/9340fd51e48576a75adc919bff65dbc4a5d4c99b
   */
  emoji: () => (Zn === void 0 && (Zn = RegExp("^(\\p{Extended_Pictographic}|\\p{Emoji_Component})+$", "u")), Zn),
  /**
   * Unused
   */
  uuid: /^[0-9a-fA-F]{8}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{4}\b-[0-9a-fA-F]{12}$/,
  /**
   * Unused
   */
  ipv4: /^(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])$/,
  ipv4Cidr: /^(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\/(3[0-2]|[12]?[0-9])$/,
  /**
   * Unused
   */
  ipv6: /^(([a-f0-9]{1,4}:){7}|::([a-f0-9]{1,4}:){0,6}|([a-f0-9]{1,4}:){1}:([a-f0-9]{1,4}:){0,5}|([a-f0-9]{1,4}:){2}:([a-f0-9]{1,4}:){0,4}|([a-f0-9]{1,4}:){3}:([a-f0-9]{1,4}:){0,3}|([a-f0-9]{1,4}:){4}:([a-f0-9]{1,4}:){0,2}|([a-f0-9]{1,4}:){5}:([a-f0-9]{1,4}:){0,1})([a-f0-9]{1,4}|(((25[0-5])|(2[0-4][0-9])|(1[0-9]{2})|([0-9]{1,2}))\.){3}((25[0-5])|(2[0-4][0-9])|(1[0-9]{2})|([0-9]{1,2})))$/,
  ipv6Cidr: /^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))\/(12[0-8]|1[01][0-9]|[1-9]?[0-9])$/,
  base64: /^([0-9a-zA-Z+/]{4})*(([0-9a-zA-Z+/]{2}==)|([0-9a-zA-Z+/]{3}=))?$/,
  base64url: /^([0-9a-zA-Z-_]{4})*(([0-9a-zA-Z-_]{2}(==)?)|([0-9a-zA-Z-_]{3}(=)?))?$/,
  nanoid: /^[a-zA-Z0-9_-]{21}$/,
  jwt: /^[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+\.[A-Za-z0-9-_]*$/
};
function to(t, e) {
  const r = {
    type: "string"
  };
  if (t.checks)
    for (const n of t.checks)
      switch (n.kind) {
        case "min":
          W(r, "minLength", typeof r.minLength == "number" ? Math.max(r.minLength, n.value) : n.value, n.message, e);
          break;
        case "max":
          W(r, "maxLength", typeof r.maxLength == "number" ? Math.min(r.maxLength, n.value) : n.value, n.message, e);
          break;
        case "email":
          switch (e.emailStrategy) {
            case "format:email":
              Pe(r, "email", n.message, e);
              break;
            case "format:idn-email":
              Pe(r, "idn-email", n.message, e);
              break;
            case "pattern:zod":
              ue(r, Oe.email, n.message, e);
              break;
          }
          break;
        case "url":
          Pe(r, "uri", n.message, e);
          break;
        case "uuid":
          Pe(r, "uuid", n.message, e);
          break;
        case "regex":
          ue(r, n.regex, n.message, e);
          break;
        case "cuid":
          ue(r, Oe.cuid, n.message, e);
          break;
        case "cuid2":
          ue(r, Oe.cuid2, n.message, e);
          break;
        case "startsWith":
          ue(r, RegExp(`^${Un(n.value, e)}`), n.message, e);
          break;
        case "endsWith":
          ue(r, RegExp(`${Un(n.value, e)}$`), n.message, e);
          break;
        case "datetime":
          Pe(r, "date-time", n.message, e);
          break;
        case "date":
          Pe(r, "date", n.message, e);
          break;
        case "time":
          Pe(r, "time", n.message, e);
          break;
        case "duration":
          Pe(r, "duration", n.message, e);
          break;
        case "length":
          W(r, "minLength", typeof r.minLength == "number" ? Math.max(r.minLength, n.value) : n.value, n.message, e), W(r, "maxLength", typeof r.maxLength == "number" ? Math.min(r.maxLength, n.value) : n.value, n.message, e);
          break;
        case "includes": {
          ue(r, RegExp(Un(n.value, e)), n.message, e);
          break;
        }
        case "ip": {
          n.version !== "v6" && Pe(r, "ipv4", n.message, e), n.version !== "v4" && Pe(r, "ipv6", n.message, e);
          break;
        }
        case "base64url":
          ue(r, Oe.base64url, n.message, e);
          break;
        case "jwt":
          ue(r, Oe.jwt, n.message, e);
          break;
        case "cidr": {
          n.version !== "v6" && ue(r, Oe.ipv4Cidr, n.message, e), n.version !== "v4" && ue(r, Oe.ipv6Cidr, n.message, e);
          break;
        }
        case "emoji":
          ue(r, Oe.emoji(), n.message, e);
          break;
        case "ulid": {
          ue(r, Oe.ulid, n.message, e);
          break;
        }
        case "base64": {
          switch (e.base64Strategy) {
            case "format:binary": {
              Pe(r, "binary", n.message, e);
              break;
            }
            case "contentEncoding:base64": {
              W(r, "contentEncoding", "base64", n.message, e);
              break;
            }
            case "pattern:zod": {
              ue(r, Oe.base64, n.message, e);
              break;
            }
          }
          break;
        }
        case "nanoid":
          ue(r, Oe.nanoid, n.message, e);
      }
  return r;
}
function Un(t, e) {
  return e.patternStrategy === "escape" ? cd(t) : t;
}
const ld = new Set("ABCDEFGHIJKLMNOPQRSTUVXYZabcdefghijklmnopqrstuvxyz0123456789");
function cd(t) {
  let e = "";
  for (let r = 0; r < t.length; r++)
    ld.has(t[r]) || (e += "\\"), e += t[r];
  return e;
}
function Pe(t, e, r, n) {
  t.format || t.anyOf?.some((s) => s.format) ? (t.anyOf || (t.anyOf = []), t.format && (t.anyOf.push({
    format: t.format,
    ...t.errorMessage && n.errorMessages && {
      errorMessage: { format: t.errorMessage.format }
    }
  }), delete t.format, t.errorMessage && (delete t.errorMessage.format, Object.keys(t.errorMessage).length === 0 && delete t.errorMessage)), t.anyOf.push({
    format: e,
    ...r && n.errorMessages && { errorMessage: { format: r } }
  })) : W(t, "format", e, r, n);
}
function ue(t, e, r, n) {
  t.pattern || t.allOf?.some((s) => s.pattern) ? (t.allOf || (t.allOf = []), t.pattern && (t.allOf.push({
    pattern: t.pattern,
    ...t.errorMessage && n.errorMessages && {
      errorMessage: { pattern: t.errorMessage.pattern }
    }
  }), delete t.pattern, t.errorMessage && (delete t.errorMessage.pattern, Object.keys(t.errorMessage).length === 0 && delete t.errorMessage)), t.allOf.push({
    pattern: Ai(e, n),
    ...r && n.errorMessages && { errorMessage: { pattern: r } }
  })) : W(t, "pattern", Ai(e, n), r, n);
}
function Ai(t, e) {
  if (!e.applyRegexFlags || !t.flags)
    return t.source;
  const r = {
    i: t.flags.includes("i"),
    m: t.flags.includes("m"),
    s: t.flags.includes("s")
    // `.` matches newlines
  }, n = r.i ? t.source.toLowerCase() : t.source;
  let s = "", i = !1, a = !1, l = !1;
  for (let c = 0; c < n.length; c++) {
    if (i) {
      s += n[c], i = !1;
      continue;
    }
    if (r.i) {
      if (a) {
        if (n[c].match(/[a-z]/)) {
          l ? (s += n[c], s += `${n[c - 2]}-${n[c]}`.toUpperCase(), l = !1) : n[c + 1] === "-" && n[c + 2]?.match(/[a-z]/) ? (s += n[c], l = !0) : s += `${n[c]}${n[c].toUpperCase()}`;
          continue;
        }
      } else if (n[c].match(/[a-z]/)) {
        s += `[${n[c]}${n[c].toUpperCase()}]`;
        continue;
      }
    }
    if (r.m) {
      if (n[c] === "^") {
        s += `(^|(?<=[\r
]))`;
        continue;
      } else if (n[c] === "$") {
        s += `($|(?=[\r
]))`;
        continue;
      }
    }
    if (r.s && n[c] === ".") {
      s += a ? `${n[c]}\r
` : `[${n[c]}\r
]`;
      continue;
    }
    s += n[c], n[c] === "\\" ? i = !0 : a && n[c] === "]" ? a = !1 : !a && n[c] === "[" && (a = !0);
  }
  try {
    new RegExp(s);
  } catch {
    return console.warn(`Could not convert regex pattern at ${e.currentPath.join("/")} to a flag-independent form! Falling back to the flag-ignorant source`), t.source;
  }
  return s;
}
function ro(t, e) {
  if (e.target === "openAi" && console.warn("Warning: OpenAI may not support records in schemas! Try an array of key-value pairs instead."), e.target === "openApi3" && t.keyType?._def.typeName === A.ZodEnum)
    return {
      type: "object",
      required: t.keyType._def.values,
      properties: t.keyType._def.values.reduce((n, s) => ({
        ...n,
        [s]: U(t.valueType._def, {
          ...e,
          currentPath: [...e.currentPath, "properties", s]
        }) ?? me(e)
      }), {}),
      additionalProperties: e.rejectedAdditionalProperties
    };
  const r = {
    type: "object",
    additionalProperties: U(t.valueType._def, {
      ...e,
      currentPath: [...e.currentPath, "additionalProperties"]
    }) ?? e.allowedAdditionalProperties
  };
  if (e.target === "openApi3")
    return r;
  if (t.keyType?._def.typeName === A.ZodString && t.keyType._def.checks?.length) {
    const { type: n, ...s } = to(t.keyType._def, e);
    return {
      ...r,
      propertyNames: s
    };
  } else {
    if (t.keyType?._def.typeName === A.ZodEnum)
      return {
        ...r,
        propertyNames: {
          enum: t.keyType._def.values
        }
      };
    if (t.keyType?._def.typeName === A.ZodBranded && t.keyType._def.type._def.typeName === A.ZodString && t.keyType._def.type._def.checks?.length) {
      const { type: n, ...s } = Ka(t.keyType._def, e);
      return {
        ...r,
        propertyNames: s
      };
    }
  }
  return r;
}
function ud(t, e) {
  if (e.mapStrategy === "record")
    return ro(t, e);
  const r = U(t.keyType._def, {
    ...e,
    currentPath: [...e.currentPath, "items", "items", "0"]
  }) || me(e), n = U(t.valueType._def, {
    ...e,
    currentPath: [...e.currentPath, "items", "items", "1"]
  }) || me(e);
  return {
    type: "array",
    maxItems: 125,
    items: {
      type: "array",
      items: [r, n],
      minItems: 2,
      maxItems: 2
    }
  };
}
function dd(t) {
  const e = t.values, n = Object.keys(t.values).filter((i) => typeof e[e[i]] != "number").map((i) => e[i]), s = Array.from(new Set(n.map((i) => typeof i)));
  return {
    type: s.length === 1 ? s[0] === "string" ? "string" : "number" : ["string", "number"],
    enum: n
  };
}
function fd(t) {
  return t.target === "openAi" ? void 0 : {
    not: me({
      ...t,
      currentPath: [...t.currentPath, "not"]
    })
  };
}
function hd(t) {
  return t.target === "openApi3" ? {
    enum: ["null"],
    nullable: !0
  } : {
    type: "null"
  };
}
const fn = {
  ZodString: "string",
  ZodNumber: "number",
  ZodBigInt: "integer",
  ZodBoolean: "boolean",
  ZodNull: "null"
};
function pd(t, e) {
  if (e.target === "openApi3")
    return wi(t, e);
  const r = t.options instanceof Map ? Array.from(t.options.values()) : t.options;
  if (r.every((n) => n._def.typeName in fn && (!n._def.checks || !n._def.checks.length))) {
    const n = r.reduce((s, i) => {
      const a = fn[i._def.typeName];
      return a && !s.includes(a) ? [...s, a] : s;
    }, []);
    return {
      type: n.length > 1 ? n : n[0]
    };
  } else if (r.every((n) => n._def.typeName === "ZodLiteral" && !n.description)) {
    const n = r.reduce((s, i) => {
      const a = typeof i._def.value;
      switch (a) {
        case "string":
        case "number":
        case "boolean":
          return [...s, a];
        case "bigint":
          return [...s, "integer"];
        case "object":
          if (i._def.value === null)
            return [...s, "null"];
        case "symbol":
        case "undefined":
        case "function":
        default:
          return s;
      }
    }, []);
    if (n.length === r.length) {
      const s = n.filter((i, a, l) => l.indexOf(i) === a);
      return {
        type: s.length > 1 ? s : s[0],
        enum: r.reduce((i, a) => i.includes(a._def.value) ? i : [...i, a._def.value], [])
      };
    }
  } else if (r.every((n) => n._def.typeName === "ZodEnum"))
    return {
      type: "string",
      enum: r.reduce((n, s) => [
        ...n,
        ...s._def.values.filter((i) => !n.includes(i))
      ], [])
    };
  return wi(t, e);
}
const wi = (t, e) => {
  const r = (t.options instanceof Map ? Array.from(t.options.values()) : t.options).map((n, s) => U(n._def, {
    ...e,
    currentPath: [...e.currentPath, "anyOf", `${s}`]
  })).filter((n) => !!n && (!e.strictUnions || typeof n == "object" && Object.keys(n).length > 0));
  return r.length ? { anyOf: r } : void 0;
};
function md(t, e) {
  if (["ZodString", "ZodNumber", "ZodBigInt", "ZodBoolean", "ZodNull"].includes(t.innerType._def.typeName) && (!t.innerType._def.checks || !t.innerType._def.checks.length))
    return e.target === "openApi3" ? {
      type: fn[t.innerType._def.typeName],
      nullable: !0
    } : {
      type: [
        fn[t.innerType._def.typeName],
        "null"
      ]
    };
  if (e.target === "openApi3") {
    const n = U(t.innerType._def, {
      ...e,
      currentPath: [...e.currentPath]
    });
    return n && "$ref" in n ? { allOf: [n], nullable: !0 } : n && { ...n, nullable: !0 };
  }
  const r = U(t.innerType._def, {
    ...e,
    currentPath: [...e.currentPath, "anyOf", "0"]
  });
  return r && { anyOf: [r, { type: "null" }] };
}
function gd(t, e) {
  const r = {
    type: "number"
  };
  if (!t.checks)
    return r;
  for (const n of t.checks)
    switch (n.kind) {
      case "int":
        r.type = "integer", Xa(r, "type", n.message, e);
        break;
      case "min":
        e.target === "jsonSchema7" ? n.inclusive ? W(r, "minimum", n.value, n.message, e) : W(r, "exclusiveMinimum", n.value, n.message, e) : (n.inclusive || (r.exclusiveMinimum = !0), W(r, "minimum", n.value, n.message, e));
        break;
      case "max":
        e.target === "jsonSchema7" ? n.inclusive ? W(r, "maximum", n.value, n.message, e) : W(r, "exclusiveMaximum", n.value, n.message, e) : (n.inclusive || (r.exclusiveMaximum = !0), W(r, "maximum", n.value, n.message, e));
        break;
      case "multipleOf":
        W(r, "multipleOf", n.value, n.message, e);
        break;
    }
  return r;
}
function vd(t, e) {
  const r = e.target === "openAi", n = {
    type: "object",
    properties: {}
  }, s = [], i = t.shape();
  for (const l in i) {
    let c = i[l];
    if (c === void 0 || c._def === void 0)
      continue;
    let f = yd(c);
    f && r && (c._def.typeName === "ZodOptional" && (c = c._def.innerType), c.isNullable() || (c = c.nullable()), f = !1);
    const d = U(c._def, {
      ...e,
      currentPath: [...e.currentPath, "properties", l],
      propertyPath: [...e.currentPath, "properties", l]
    });
    d !== void 0 && (n.properties[l] = d, f || s.push(l));
  }
  s.length && (n.required = s);
  const a = bd(t, e);
  return a !== void 0 && (n.additionalProperties = a), n;
}
function bd(t, e) {
  if (t.catchall._def.typeName !== "ZodNever")
    return U(t.catchall._def, {
      ...e,
      currentPath: [...e.currentPath, "additionalProperties"]
    });
  switch (t.unknownKeys) {
    case "passthrough":
      return e.allowedAdditionalProperties;
    case "strict":
      return e.rejectedAdditionalProperties;
    case "strip":
      return e.removeAdditionalStrategy === "strict" ? e.allowedAdditionalProperties : e.rejectedAdditionalProperties;
  }
}
function yd(t) {
  try {
    return t.isOptional();
  } catch {
    return !0;
  }
}
const _d = (t, e) => {
  if (e.currentPath.toString() === e.propertyPath?.toString())
    return U(t.innerType._def, e);
  const r = U(t.innerType._def, {
    ...e,
    currentPath: [...e.currentPath, "anyOf", "1"]
  });
  return r ? {
    anyOf: [
      {
        not: me(e)
      },
      r
    ]
  } : me(e);
}, Ad = (t, e) => {
  if (e.pipeStrategy === "input")
    return U(t.in._def, e);
  if (e.pipeStrategy === "output")
    return U(t.out._def, e);
  const r = U(t.in._def, {
    ...e,
    currentPath: [...e.currentPath, "allOf", "0"]
  }), n = U(t.out._def, {
    ...e,
    currentPath: [...e.currentPath, "allOf", r ? "1" : "0"]
  });
  return {
    allOf: [r, n].filter((s) => s !== void 0)
  };
};
function wd(t, e) {
  return U(t.type._def, e);
}
function kd(t, e) {
  const n = {
    type: "array",
    uniqueItems: !0,
    items: U(t.valueType._def, {
      ...e,
      currentPath: [...e.currentPath, "items"]
    })
  };
  return t.minSize && W(n, "minItems", t.minSize.value, t.minSize.message, e), t.maxSize && W(n, "maxItems", t.maxSize.value, t.maxSize.message, e), n;
}
function $d(t, e) {
  return t.rest ? {
    type: "array",
    minItems: t.items.length,
    items: t.items.map((r, n) => U(r._def, {
      ...e,
      currentPath: [...e.currentPath, "items", `${n}`]
    })).reduce((r, n) => n === void 0 ? r : [...r, n], []),
    additionalItems: U(t.rest._def, {
      ...e,
      currentPath: [...e.currentPath, "additionalItems"]
    })
  } : {
    type: "array",
    minItems: t.items.length,
    maxItems: t.items.length,
    items: t.items.map((r, n) => U(r._def, {
      ...e,
      currentPath: [...e.currentPath, "items", `${n}`]
    })).reduce((r, n) => n === void 0 ? r : [...r, n], [])
  };
}
function Sd(t) {
  return {
    not: me(t)
  };
}
function xd(t) {
  return me(t);
}
const Cd = (t, e) => U(t.innerType._def, e), Td = (t, e, r) => {
  switch (e) {
    case A.ZodString:
      return to(t, r);
    case A.ZodNumber:
      return gd(t, r);
    case A.ZodObject:
      return vd(t, r);
    case A.ZodBigInt:
      return Qu(t, r);
    case A.ZodBoolean:
      return Ku();
    case A.ZodDate:
      return eo(t, r);
    case A.ZodUndefined:
      return Sd(r);
    case A.ZodNull:
      return hd(r);
    case A.ZodArray:
      return Xu(t, r);
    case A.ZodUnion:
    case A.ZodDiscriminatedUnion:
      return pd(t, r);
    case A.ZodIntersection:
      return ad(t, r);
    case A.ZodTuple:
      return $d(t, r);
    case A.ZodRecord:
      return ro(t, r);
    case A.ZodLiteral:
      return od(t, r);
    case A.ZodEnum:
      return sd(t);
    case A.ZodNativeEnum:
      return dd(t);
    case A.ZodNullable:
      return md(t, r);
    case A.ZodOptional:
      return _d(t, r);
    case A.ZodMap:
      return ud(t, r);
    case A.ZodSet:
      return kd(t, r);
    case A.ZodLazy:
      return () => t.getter()._def;
    case A.ZodPromise:
      return wd(t, r);
    case A.ZodNaN:
    case A.ZodNever:
      return fd(r);
    case A.ZodEffects:
      return nd(t, r);
    case A.ZodAny:
      return me(r);
    case A.ZodUnknown:
      return xd(r);
    case A.ZodDefault:
      return rd(t, r);
    case A.ZodBranded:
      return Ka(t, r);
    case A.ZodReadonly:
      return Cd(t, r);
    case A.ZodCatch:
      return ed(t, r);
    case A.ZodPipeline:
      return Ad(t, r);
    case A.ZodFunction:
    case A.ZodVoid:
    case A.ZodSymbol:
      return;
    default:
      return /* @__PURE__ */ ((n) => {
      })();
  }
};
function U(t, e, r = !1) {
  const n = e.seen.get(t);
  if (e.override) {
    const l = e.override?.(t, e, n, r);
    if (l !== Yu)
      return l;
  }
  if (n && !r) {
    const l = Ed(n, e);
    if (l !== void 0)
      return l;
  }
  const s = { def: t, path: e.currentPath, jsonSchema: void 0 };
  e.seen.set(t, s);
  const i = Td(t, t.typeName, e), a = typeof i == "function" ? U(i(), e) : i;
  if (a && Od(t, e, a), e.postProcess) {
    const l = e.postProcess(a, t, e);
    return s.jsonSchema = a, l;
  }
  return s.jsonSchema = a, a;
}
const Ed = (t, e) => {
  switch (e.$refStrategy) {
    case "root":
      return { $ref: t.path.join("/") };
    case "relative":
      return { $ref: Qa(e.currentPath, t.path) };
    case "none":
    case "seen":
      return t.path.length < e.currentPath.length && t.path.every((r, n) => e.currentPath[n] === r) ? (console.warn(`Recursive reference detected at ${e.currentPath.join("/")}! Defaulting to any`), me(e)) : e.$refStrategy === "seen" ? me(e) : void 0;
  }
}, Od = (t, e, r) => (t.description && (r.description = t.description, e.markdownDescription && (r.markdownDescription = t.description)), r), Vn = (t, e) => {
  const r = Ju(e);
  let n = typeof e == "object" && e.definitions ? Object.entries(e.definitions).reduce((c, [f, d]) => ({
    ...c,
    [f]: U(d._def, {
      ...r,
      currentPath: [...r.basePath, r.definitionPath, f]
    }, !0) ?? me(r)
  }), {}) : void 0;
  const s = typeof e == "string" ? e : e?.nameStrategy === "title" ? void 0 : e?.name, i = U(t._def, s === void 0 ? r : {
    ...r,
    currentPath: [...r.basePath, r.definitionPath, s]
  }, !1) ?? me(r), a = typeof e == "object" && e.name !== void 0 && e.nameStrategy === "title" ? e.name : void 0;
  a !== void 0 && (i.title = a), r.flags.hasReferencedOpenAiAnyType && (n || (n = {}), n[r.openAiAnyTypeName] || (n[r.openAiAnyTypeName] = {
    // Skipping "object" as no properties can be defined and additionalProperties must be "false"
    type: ["string", "number", "integer", "boolean", "array", "null"],
    items: {
      $ref: r.$refStrategy === "relative" ? "1" : [
        ...r.basePath,
        r.definitionPath,
        r.openAiAnyTypeName
      ].join("/")
    }
  }));
  const l = s === void 0 ? n ? {
    ...i,
    [r.definitionPath]: n
  } : i : {
    $ref: [
      ...r.$refStrategy === "relative" ? [] : r.basePath,
      r.definitionPath,
      s
    ].join("/"),
    [r.definitionPath]: {
      ...n,
      [s]: i
    }
  };
  return r.target === "jsonSchema7" ? l.$schema = "http://json-schema.org/draft-07/schema#" : (r.target === "jsonSchema2019-09" || r.target === "openAi") && (l.$schema = "https://json-schema.org/draft/2019-09/schema#"), r.target === "openAi" && ("anyOf" in l || "oneOf" in l || "allOf" in l || "type" in l && Array.isArray(l.type)) && console.warn("Warning: OpenAI may not support schemas with unions as roots! Try wrapping it in an object property."), l;
};
class Pd {
  /**
   * Creates a new message processor.
   *
   * @param catalogs A list of available catalogs.
   * @param actionHandler A global handler for actions from all surfaces.
   */
  constructor(e, r) {
    this.catalogs = e, this.actionHandler = r, this.model = new Hu(), this.actionHandler && this.model.onAction.subscribe(this.actionHandler);
  }
  /**
   * Generates the a2uiClientCapabilities object for the current processor.
   *
   * @param options Configuration for capability generation.
   * @returns The capabilities object.
   */
  getClientCapabilities(e) {
    const r = {
      "v0.9": {
        supportedCatalogIds: this.catalogs.map((n) => n.id)
      }
    };
    return e?.includeInlineCatalogs && (r["v0.9"].inlineCatalogs = this.catalogs.map((n) => this.generateInlineCatalog(n))), r;
  }
  generateInlineCatalog(e) {
    const r = {};
    for (const [i, a] of e.components.entries()) {
      const l = Vn(a.schema, {
        target: "jsonSchema2019-09"
      });
      this.processRefs(l), r[i] = {
        allOf: [
          { $ref: "common_types.json#/$defs/ComponentCommon" },
          {
            properties: {
              component: { const: i },
              ...l.properties
            },
            required: ["component", ...l.required || []]
          }
        ]
      };
    }
    const n = [];
    for (const i of e.functions.values()) {
      const a = Vn(i.schema, {
        target: "jsonSchema2019-09"
      });
      this.processRefs(a), n.push({
        name: i.name,
        description: i.schema.description,
        returnType: i.returnType,
        parameters: a
      });
    }
    let s;
    if (e.themeSchema) {
      const i = Vn(e.themeSchema, {
        target: "jsonSchema2019-09"
      });
      this.processRefs(i), s = i.properties;
    }
    return {
      catalogId: e.id,
      components: r,
      functions: n.length > 0 ? n : void 0,
      theme: s
    };
  }
  processRefs(e) {
    if (!(typeof e != "object" || e === null)) {
      if (typeof e.description == "string" && e.description.startsWith("REF:")) {
        const r = e.description.substring(4).split("|"), n = r[0], s = r[1] || "";
        for (const i of Object.keys(e))
          delete e[i];
        e.$ref = n, s && (e.description = s);
        return;
      }
      if (Array.isArray(e))
        for (const r of e)
          this.processRefs(r);
      else
        for (const r of Object.keys(e))
          this.processRefs(e[r]);
    }
  }
  /**
   * Returns the aggregated data model for all surfaces that have 'sendDataModel' enabled.
   */
  getClientDataModel() {
    const e = {};
    for (const r of this.model.surfacesMap.values())
      r.sendDataModel && (e[r.id] = r.dataModel.get("/"));
    if (Object.keys(e).length !== 0)
      return {
        version: "v0.9",
        surfaces: e
      };
  }
  /**
   * Subscribes to surface creation events.
   */
  onSurfaceCreated(e) {
    return this.model.onSurfaceCreated.subscribe(e);
  }
  /**
   * Subscribes to surface deletion events.
   */
  onSurfaceDeleted(e) {
    return this.model.onSurfaceDeleted.subscribe(e);
  }
  /**
   * Processes a list of messages or a messages wrapper.
   *
   * @param messages The messages or messages wrapper to process.
   */
  processMessages(e) {
    const r = Array.isArray(e) ? e : e.messages;
    for (const n of r)
      this.processMessage(n);
  }
  processMessage(e) {
    const r = [
      "createSurface",
      "updateComponents",
      "updateDataModel",
      "deleteSurface"
    ].filter((n) => n in e);
    if (r.length > 1)
      throw new In(`Message contains multiple update types: ${r.join(", ")}.`);
    if ("createSurface" in e) {
      this.processCreateSurfaceMessage(e);
      return;
    }
    if ("deleteSurface" in e) {
      this.processDeleteSurfaceMessage(e);
      return;
    }
    if ("updateComponents" in e) {
      this.processUpdateComponentsMessage(e);
      return;
    }
    if ("updateDataModel" in e) {
      this.processUpdateDataModelMessage(e);
      return;
    }
  }
  processCreateSurfaceMessage(e) {
    const r = e.createSurface, { surfaceId: n, catalogId: s, theme: i, sendDataModel: a } = r, l = this.catalogs.find((f) => f.id === s);
    if (!l)
      throw new Wt(`Catalog not found: ${s}`);
    if (this.model.getSurface(n))
      throw new Wt(`Surface ${n} already exists.`);
    const c = new qu(n, l, i, a ?? !1);
    this.model.addSurface(c);
  }
  processDeleteSurfaceMessage(e) {
    const r = e.deleteSurface;
    r.surfaceId && this.model.deleteSurface(r.surfaceId);
  }
  processUpdateComponentsMessage(e) {
    const r = e.updateComponents;
    if (!r.surfaceId)
      return;
    const n = this.model.getSurface(r.surfaceId);
    if (!n)
      throw new Wt(`Surface not found for message: ${r.surfaceId}`);
    for (const s of r.components) {
      const { id: i, component: a, ...l } = s;
      if (!i)
        throw new In(`Component '${a}' is missing an 'id'.`);
      const c = n.componentsModel.get(i);
      if (c)
        if (a && a !== c.type) {
          n.componentsModel.removeComponent(i);
          const f = new yi(i, a, l);
          n.componentsModel.addComponent(f);
        } else
          c.properties = l;
      else {
        if (!a)
          throw new In(`Cannot create component ${i} without a type.`);
        const f = new yi(i, a, l);
        n.componentsModel.addComponent(f);
      }
    }
  }
  processUpdateDataModelMessage(e) {
    const r = e.updateDataModel;
    if (!r.surfaceId)
      return;
    const n = this.model.getSurface(r.surfaceId);
    if (!n)
      throw new Wt(`Surface not found for message: ${r.surfaceId}`);
    const s = r.path || "/", i = r.value;
    n.dataModel.set(s, i);
  }
  /**
   * Resolves a relative path against a context path.
   *
   * @param path The path to resolve.
   * @param contextPath The base path (optional).
   */
  resolvePath(e, r) {
    return e.startsWith("/") ? e : r ? `${r.endsWith("/") ? r : `${r}/`}${e}` : `/${e}`;
  }
}
class Es {
  /**
   * Initializes a new DataContext.
   *
   * @param surface The surface model this context belongs to.
   * @param path The absolute path in the DataModel that this context is scoped to (its "current working directory").
   */
  constructor(e, r) {
    this.surface = e, this.path = r, this.dataModel = e.dataModel, this.functionInvoker = e.catalog.invoker;
  }
  /**
   * Mutates the underlying DataModel at the specified path.
   *
   * This is the primary method for components to push state changes (e.g. user input)
   * back up to the global model.
   *
   * @param path A JSON pointer path. If relative, it is resolved against this context's `path`.
   * @param value The new value to store in the DataModel.
   */
  set(e, r) {
    const n = this.resolvePath(e);
    this.dataModel.set(n, r);
  }
  /**
   * Synchronously evaluates a `DynamicValue` (a literal, a path binding, or a function call)
   * into its concrete runtime value.
   *
   * **Note:** This method evaluates the value *once* at the current moment in time.
   * It does not create any reactive subscriptions. If the underlying data changes later,
   * this result will not automatically update. Use `subscribeDynamicValue` for reactive updates.
   *
   * @param value The DynamicValue object from the A2UI JSON payload.
   * @returns The synchronously resolved value.
   */
  resolveDynamicValue(e) {
    if (e === null || typeof e != "object" || Array.isArray(e))
      return e;
    if ("path" in e) {
      const r = this.resolvePath(e.path);
      return this.dataModel.get(r);
    }
    if ("call" in e) {
      const r = e, n = {};
      for (const [a, l] of Object.entries(r.args))
        n[a] = this.resolveDynamicValue(l);
      const s = new AbortController(), i = this.evaluateFunctionReactive(r.call, n, s.signal);
      return i === void 0 ? void 0 : fs(i) ? i.peek() : i;
    }
    return e;
  }
  /**
   * Reactively listens to changes in a `DynamicValue`.
   *
   * This is the core reactive binding mechanism. Whenever the underlying data changes
   * (or if a function call's dependencies change), the `onChange` callback will be fired
   * with the freshly evaluated result.
   *
   * @template V The expected type of the resolved value.
   * @param value The DynamicValue to evaluate and observe.
   * @param onChange A callback fired whenever the evaluated result changes.
   * @returns A `DataSubscription` containing the initial synchronously-resolved value, along with an `unsubscribe` method to clean up the listener.
   */
  subscribeDynamicValue(e, r) {
    const n = this.resolveSignal(e);
    let s = !0, i = n.peek();
    const a = Tr(() => {
      const l = n.value;
      i = l, s || r(l);
    });
    return s = !1, {
      get value() {
        return i;
      },
      unsubscribe: () => {
        a(), n.unsubscribe && n.unsubscribe();
      }
    };
  }
  /**
   * Returns a Preact Signal representing the reactive dynamic value.
   *
   * This method recursively resolves any nested path bindings or function calls into a
   * single, reactive `Signal`. Any changes to the underlying data or function dependencies
   * will cause this signal's value to update.
   *
   * @param value The DynamicValue to evaluate and observe.
   * @returns A Preact Signal containing the reactive result of the evaluation.
   */
  resolveSignal(e) {
    if (typeof e != "object" || e === null || Array.isArray(e))
      return wr(e);
    if ("path" in e) {
      const r = this.resolvePath(e.path);
      return this.dataModel.getSignal(r);
    }
    if ("call" in e) {
      const r = e, n = {};
      for (const [d, u] of Object.entries(r.args))
        n[d] = this.resolveSignal(u);
      if (Object.keys(n).length === 0) {
        const d = new AbortController(), u = this.evaluateFunctionReactive(r.call, {}, d.signal), o = u instanceof ce ? u : wr(u);
        return o.unsubscribe = () => d.abort(), o;
      }
      const s = Object.keys(n), i = wr(void 0);
      let a, l;
      const c = Ya(() => {
        const d = {};
        for (let u = 0; u < s.length; u++)
          d[s[u]] = n[s[u]].value;
        return d;
      }), f = Tr(() => {
        try {
          const d = c.value;
          a && a.abort(), l && (l(), l = void 0), a = new AbortController();
          const u = this.evaluateFunctionReactive(r.call, d, a.signal);
          fs(u) ? l = Tr(() => {
            i.value = u.value;
          }) : i.value = u;
        } catch (d) {
          this.dispatchExpressionError(d, r.call), i.value = void 0;
        }
      });
      return i.unsubscribe = () => {
        f(), l && l(), a && a.abort();
        for (let d = 0; d < s.length; d++) {
          const u = n[s[d]];
          u.unsubscribe && u.unsubscribe();
        }
      }, i;
    }
    return wr(e);
  }
  /**
   * Resolves an action by evaluating its top-level dynamic values.
   *
   * For event actions, it resolves each value in the context map.
   * For function call actions, it evaluates the call.
   *
   * This is non-recursive: it only resolves one level deep for the context record,
   * in accordance with the schema specification that requires values to be single
   * DynamicValue types and prevents arbitrary nesting.
   */
  resolveAction(e) {
    if ("event" in e) {
      const r = {};
      if (e.event.context)
        for (const [n, s] of Object.entries(e.event.context))
          r[n] = this.resolveDynamicValue(s);
      return {
        event: {
          ...e.event,
          context: r
        }
      };
    }
    return "functionCall" in e ? this.resolveDynamicValue(e.functionCall) : e;
  }
  evaluateFunctionReactive(e, r, n) {
    try {
      return this.functionInvoker(e, r, this, n);
    } catch (s) {
      this.dispatchExpressionError(s, e);
      return;
    }
  }
  dispatchExpressionError(e, r) {
    if (e?.name === "ZodError" || e instanceof Re) {
      const n = new Me(`Validation failed for function '${r}': ${e.message}`, r, e.errors ?? e.issues);
      this.surface.dispatchError({
        code: "EXPRESSION_ERROR",
        message: n.message,
        expression: r,
        details: n.details
      });
    } else e instanceof Me ? this.surface.dispatchError({
      code: "EXPRESSION_ERROR",
      message: e.message,
      expression: e.expression,
      details: e.details
    }) : this.surface.dispatchError({
      code: "EXPRESSION_ERROR",
      message: e.message ?? `An unexpected error occurred in function ${r}.`,
      expression: r,
      details: { stack: e.stack }
    });
  }
  /**
   * Creates a new, child `DataContext` scoped to a deeper path.
   *
   * This is used when a component (like a List or a Card) wants to provide a targeted
   * data scope for its children, so children can use relative paths like `./title`.
   *
   * @param relativePath The path relative to the *current* context's path.
   * @returns A new `DataContext` instance pointing to the resolved absolute path.
   */
  nested(e) {
    const r = this.resolvePath(e);
    return new Es(this.surface, r);
  }
  resolvePath(e) {
    if (e.startsWith("/"))
      return e;
    if (e === "" || e === ".")
      return this.path;
    let r = this.path;
    return r.endsWith("/") && r.length > 1 && (r = r.slice(0, -1)), r === "/" && (r = ""), `${r}/${e}`;
  }
}
class no {
  /**
   * Creates a new component context.
   *
   * @param surface The surface model the component belongs to.
   * @param componentId The ID of the component.
   * @param dataModelBasePath The base path for data model access (default: '/').
   */
  constructor(e, r, n = "/") {
    const s = e.componentsModel.get(r);
    if (!s)
      throw new Wt(`Component not found: ${r}`);
    this.componentModel = s, this.surfaceComponents = e.componentsModel, this.theme = e.theme, this.dataContext = new Es(e, n), this._actionDispatcher = (i) => e.dispatchAction(i, this.componentModel.id);
  }
  /**
   * Dispatches an action from the component.
   *
   * @param action The action to dispatch.
   */
  dispatchAction(e) {
    return this._actionDispatcher(e);
  }
}
function Dd(t) {
  return ps(t);
}
function ps(t, e) {
  let r = t;
  for (; r._def.typeName === "ZodOptional" || r._def.typeName === "ZodNullable" || r._def.typeName === "ZodDefault"; )
    r = r._def.innerType;
  if (e === "checks")
    return { type: "CHECKABLE" };
  if (r._def.typeName === "ZodUnion") {
    const n = r._def.options;
    if (n.some((l) => l._def.typeName === "ZodObject" && l._def.shape().event))
      return { type: "ACTION" };
    if (n.some((l) => l._def.typeName === "ZodObject" && l._def.shape().path && !l._def.shape().componentId))
      return { type: "DYNAMIC" };
    if (n.some((l) => l._def.typeName === "ZodObject" && l._def.shape().componentId && l._def.shape().path))
      return { type: "STRUCTURAL" };
  } else r._def.typeName;
  if (r._def.typeName === "ZodArray")
    return {
      type: "ARRAY",
      element: ps(r._def.type)
    };
  if (r._def.typeName === "ZodObject") {
    const n = {}, s = r._def.shape();
    for (const [i, a] of Object.entries(s))
      n[i] = ps(a, i);
    return { type: "OBJECT", shape: n };
  }
  return { type: "STATIC" };
}
class Nd {
  constructor(e, r) {
    this.dataListeners = [], this.propsListeners = [], this.currentProps = {}, this.isConnected = !1, this.context = e, this.behaviorTree = Dd(r), this.behaviorTree.type !== "OBJECT" && (this.behaviorTree = { type: "OBJECT", shape: {} }), this.resolveInitialProps();
  }
  resolveInitialProps() {
    const e = this.context.componentModel.properties, r = this.resolveAndBind(e, this.behaviorTree, [], !0);
    this.currentProps = { ...this.currentProps, ...r };
  }
  connect() {
    if (this.isConnected)
      return;
    this.isConnected = !0;
    const e = this.context.componentModel.onUpdated.subscribe(() => {
      this.rebuildAllBindings();
    });
    this.compUnsub = () => e.unsubscribe(), this.rebuildAllBindings();
  }
  rebuildAllBindings() {
    this.dataListeners.forEach((n) => n()), this.dataListeners = [];
    const e = this.context.componentModel.properties, r = this.resolveAndBind(e, this.behaviorTree, [], !1);
    this.currentProps = { ...this.currentProps, ...r }, this.notify();
  }
  resolveAndBind(e, r, n, s) {
    if (e == null)
      return e;
    switch (r.type) {
      case "DYNAMIC": {
        const i = this.context.dataContext.subscribeDynamicValue(e, (a) => {
          this.updateDeepValue(n, a), this.notify();
        });
        return s ? i.unsubscribe() : this.dataListeners.push(() => i.unsubscribe()), i.value;
      }
      case "ACTION":
        return () => {
          const i = (a) => {
            if (typeof a != "object" || a === null)
              return a;
            if ("path" in a || "call" in a)
              return this.context.dataContext.resolveDynamicValue(a);
            if (Array.isArray(a))
              return a.map(i);
            const l = {};
            for (const [c, f] of Object.entries(a))
              l[c] = i(f);
            return l;
          };
          this.context.dispatchAction(i(e));
        };
      case "STRUCTURAL": {
        if (e && typeof e == "object" && e.path && e.componentId) {
          const i = this.context.dataContext.subscribeDynamicValue({ path: e.path }, (c) => {
            const f = Array.isArray(c) ? c : [], d = this.context.dataContext.nested(e.path), u = f.map((o, b) => ({
              id: e.componentId,
              basePath: d.nested(String(b)).path
            }));
            this.updateDeepValue(n, u), this.notify();
          });
          s ? i.unsubscribe() : this.dataListeners.push(() => i.unsubscribe());
          const a = Array.isArray(i.value) ? i.value : [], l = this.context.dataContext.nested(e.path);
          return a.map((c, f) => ({
            id: e.componentId,
            basePath: l.nested(String(f)).path
          }));
        }
        return e;
      }
      case "CHECKABLE": {
        const i = Array.isArray(e) ? e : [], a = i.map(() => ({ valid: !0, message: "" })), l = n.slice(0, -1), c = () => {
          const d = a.filter((u) => !u.valid).map((u) => u.message);
          this.updateDeepValue([...l, "isValid"], d.length === 0), this.updateDeepValue([...l, "validationErrors"], d), this.notify();
        };
        i.forEach((d, u) => {
          const o = d.condition || d, b = d.message || "Validation failed";
          a[u].message = b;
          const v = this.context.dataContext.subscribeDynamicValue(o, (g) => {
            a[u].valid = !!g, c();
          });
          s ? v.unsubscribe() : this.dataListeners.push(() => v.unsubscribe()), a[u].valid = !!v.value;
        });
        const f = a.filter((d) => !d.valid).map((d) => d.message);
        return this.updateDeepValue([...l, "isValid"], f.length === 0), this.updateDeepValue([...l, "validationErrors"], f), e;
      }
      case "STATIC":
        return e;
      case "ARRAY":
        return Array.isArray(e) ? e.map((i, a) => this.resolveAndBind(i, r.element, [...n, a.toString()], s)) : e;
      case "OBJECT": {
        if (typeof e != "object")
          return e;
        const i = {};
        for (const [a, l] of Object.entries(e)) {
          const c = r.shape[a] || { type: "STATIC" };
          i[a] = this.resolveAndBind(l, c, [...n, a], s);
        }
        for (const [a, l] of Object.entries(r.shape))
          if (l.type === "DYNAMIC") {
            const c = `set${a.charAt(0).toUpperCase() + a.slice(1)}`, f = e[a];
            i[c] = (d) => {
              f && typeof f == "object" && "path" in f && this.context.dataContext.set(f.path, d);
            };
          }
        return i;
      }
    }
  }
  updateDeepValue(e, r) {
    this.currentProps = this.cloneAndUpdate(this.currentProps, e, r);
  }
  cloneAndUpdate(e, r, n) {
    if (r.length === 0)
      return n;
    const [s, ...i] = r;
    if (Array.isArray(e)) {
      const a = [...e];
      return a[Number(s)] = this.cloneAndUpdate(a[Number(s)], i, n), a;
    } else
      return {
        ...e || {},
        [s]: this.cloneAndUpdate((e || {})[s], i, n)
      };
  }
  dispose() {
    this.isConnected && (this.isConnected = !1, this.dataListeners.forEach((e) => e()), this.dataListeners = [], this.compUnsub && (this.compUnsub(), this.compUnsub = void 0));
  }
  notify() {
    this.propsListeners.forEach((e) => e(this.currentProps));
  }
  subscribe(e) {
    return this.propsListeners.length === 0 && this.connect(), this.propsListeners.push(e), {
      unsubscribe: () => {
        this.propsListeners = this.propsListeners.filter((r) => r !== e), this.propsListeners.length === 0 && this.dispose();
      }
    };
  }
  get snapshot() {
    return this.currentProps;
  }
}
const Fr = k({
  path: O().describe("A JSON Pointer path to a value in the data model.")
}).describe("REF:common_types.json#/$defs/DataBinding|A JSON Pointer path to a value in the data model."), cr = k({
  call: O().describe("The name of the function to call."),
  args: _n(be()).describe("Arguments passed to the function."),
  returnType: le(["string", "number", "boolean", "array", "object", "any", "void"]).default("boolean")
}).describe("REF:common_types.json#/$defs/FunctionCall|Invokes a named function on the client."), so = Se([et(), Fr, cr]).describe("REF:common_types.json#/$defs/DynamicBoolean|A boolean value that can be a literal, a path, or a function call returning a boolean."), H = Se([
  O(),
  Fr,
  // FunctionCall returning string (simplified schema for Zod, stricter in JSON Schema)
  cr
]).describe("REF:common_types.json#/$defs/DynamicString|Represents a string"), io = Se([sr(), Fr, cr]).describe("REF:common_types.json#/$defs/DynamicNumber|Represents a value that can be either a literal number, a path to a number in the data model, or a function call returning a number."), jd = Se([tt(O()), Fr, cr]).describe("REF:common_types.json#/$defs/DynamicStringList|Represents a value that can be either a literal array of strings, a path to a string array in the data model, or a function call returning a string array."), Rd = Se([
  O(),
  sr(),
  et(),
  tt(be()),
  Fr,
  cr
]).describe("REF:common_types.json#/$defs/DynamicValue|A value that can be a literal, a path, or a function call returning any type."), ft = O().describe("REF:common_types.json#/$defs/ComponentId|The unique identifier for a component."), Os = Se([
  tt(ft).describe("A static list of child component IDs."),
  k({
    componentId: ft,
    path: O().describe("The path to the list of component property objects in the data model.")
  }).describe("A template for generating a dynamic list of children.")
]).describe("REF:common_types.json#/$defs/ChildList"), ao = Se([
  k({
    event: k({
      name: O(),
      context: _n(Rd).optional()
    })
  }).describe("Triggers a server-side event."),
  k({
    functionCall: cr
  }).describe("Executes a local client-side function.")
]).describe("REF:common_types.json#/$defs/Action"), Ld = k({
  condition: so,
  message: O().describe("The error message to display if the check fails.")
}).describe("REF:common_types.json#/$defs/CheckRule|A check rule consisting of a condition and an error message."), ur = k({
  checks: tt(Ld).optional().describe("A list of checks to perform.")
}).describe("REF:common_types.json#/$defs/Checkable|Properties for components that support client-side checks."), Md = k({
  label: H.optional().describe("REF:common_types.json#/$defs/DynamicString|A short string used by assistive technologies to convey the purpose of an element."),
  description: H.optional().describe("REF:common_types.json#/$defs/DynamicString|Additional information provided by assistive technologies about an element.")
}).describe("REF:common_types.json#/$defs/AccessibilityAttributes|Attributes to enhance accessibility.");
k({
  component: O().describe("The type name of the component."),
  id: ft.optional(),
  weight: sr().optional()
}).passthrough().describe("A generic A2UI component definition.");
class Y {
  /**
   * Initializes the controller, binding it to the given Lit element and API schema.
   *
   * @param host The A2uiLitElement acting as the component host.
   * @param api The A2UI component API defining the schema for this element.
   */
  constructor(e, r) {
    this.host = e, this.binder = new Nd(this.host.context, r.schema), this.props = this.binder.snapshot, this.host.addController(this), this.host.isConnected && this.hostConnected();
  }
  /**
   * Subscribes to the GenericBinder updates when the host connects.
   *
   * Triggers a request update on the host element when new props are received.
   */
  hostConnected() {
    this.subscription || (this.subscription = this.binder.subscribe((e) => {
      this.props = e, this.host.requestUpdate();
    }));
  }
  /**
   * Unsubscribes from the GenericBinder updates when the host disconnects.
   */
  hostDisconnected() {
    this.subscription?.unsubscribe(), this.subscription = void 0;
  }
  /**
   * Disposes the underlying GenericBinder to clean up resources from the context.
   */
  dispose() {
    this.binder.dispose();
  }
}
/**
 * @license
 * Copyright 2019 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const qr = globalThis, Ps = qr.ShadowRoot && (qr.ShadyCSS === void 0 || qr.ShadyCSS.nativeShadow) && "adoptedStyleSheets" in Document.prototype && "replace" in CSSStyleSheet.prototype, oo = Symbol(), ki = /* @__PURE__ */ new WeakMap();
let Fd = class {
  constructor(e, r, n) {
    if (this._$cssResult$ = !0, n !== oo) throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");
    this.cssText = e, this.t = r;
  }
  get styleSheet() {
    let e = this.o;
    const r = this.t;
    if (Ps && e === void 0) {
      const n = r !== void 0 && r.length === 1;
      n && (e = ki.get(r)), e === void 0 && ((this.o = e = new CSSStyleSheet()).replaceSync(this.cssText), n && ki.set(r, e));
    }
    return e;
  }
  toString() {
    return this.cssText;
  }
};
const Id = (t) => new Fd(typeof t == "string" ? t : t + "", void 0, oo), zd = (t, e) => {
  if (Ps) t.adoptedStyleSheets = e.map((r) => r instanceof CSSStyleSheet ? r : r.styleSheet);
  else for (const r of e) {
    const n = document.createElement("style"), s = qr.litNonce;
    s !== void 0 && n.setAttribute("nonce", s), n.textContent = r.cssText, t.appendChild(n);
  }
}, $i = Ps ? (t) => t : (t) => t instanceof CSSStyleSheet ? ((e) => {
  let r = "";
  for (const n of e.cssRules) r += n.cssText;
  return Id(r);
})(t) : t;
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const { is: Zd, defineProperty: Ud, getOwnPropertyDescriptor: Vd, getOwnPropertyNames: Wd, getOwnPropertySymbols: Bd, getPrototypeOf: qd } = Object, $n = globalThis, Si = $n.trustedTypes, Hd = Si ? Si.emptyScript : "", Yd = $n.reactiveElementPolyfillSupport, Er = (t, e) => t, hn = { toAttribute(t, e) {
  switch (e) {
    case Boolean:
      t = t ? Hd : null;
      break;
    case Object:
    case Array:
      t = t == null ? t : JSON.stringify(t);
  }
  return t;
}, fromAttribute(t, e) {
  let r = t;
  switch (e) {
    case Boolean:
      r = t !== null;
      break;
    case Number:
      r = t === null ? null : Number(t);
      break;
    case Object:
    case Array:
      try {
        r = JSON.parse(t);
      } catch {
        r = null;
      }
  }
  return r;
} }, Ds = (t, e) => !Zd(t, e), xi = { attribute: !0, type: String, converter: hn, reflect: !1, useDefault: !1, hasChanged: Ds };
Symbol.metadata ??= Symbol("metadata"), $n.litPropertyMetadata ??= /* @__PURE__ */ new WeakMap();
let Zt = class extends HTMLElement {
  static addInitializer(e) {
    this._$Ei(), (this.l ??= []).push(e);
  }
  static get observedAttributes() {
    return this.finalize(), this._$Eh && [...this._$Eh.keys()];
  }
  static createProperty(e, r = xi) {
    if (r.state && (r.attribute = !1), this._$Ei(), this.prototype.hasOwnProperty(e) && ((r = Object.create(r)).wrapped = !0), this.elementProperties.set(e, r), !r.noAccessor) {
      const n = Symbol(), s = this.getPropertyDescriptor(e, n, r);
      s !== void 0 && Ud(this.prototype, e, s);
    }
  }
  static getPropertyDescriptor(e, r, n) {
    const { get: s, set: i } = Vd(this.prototype, e) ?? { get() {
      return this[r];
    }, set(a) {
      this[r] = a;
    } };
    return { get: s, set(a) {
      const l = s?.call(this);
      i?.call(this, a), this.requestUpdate(e, l, n);
    }, configurable: !0, enumerable: !0 };
  }
  static getPropertyOptions(e) {
    return this.elementProperties.get(e) ?? xi;
  }
  static _$Ei() {
    if (this.hasOwnProperty(Er("elementProperties"))) return;
    const e = qd(this);
    e.finalize(), e.l !== void 0 && (this.l = [...e.l]), this.elementProperties = new Map(e.elementProperties);
  }
  static finalize() {
    if (this.hasOwnProperty(Er("finalized"))) return;
    if (this.finalized = !0, this._$Ei(), this.hasOwnProperty(Er("properties"))) {
      const r = this.properties, n = [...Wd(r), ...Bd(r)];
      for (const s of n) this.createProperty(s, r[s]);
    }
    const e = this[Symbol.metadata];
    if (e !== null) {
      const r = litPropertyMetadata.get(e);
      if (r !== void 0) for (const [n, s] of r) this.elementProperties.set(n, s);
    }
    this._$Eh = /* @__PURE__ */ new Map();
    for (const [r, n] of this.elementProperties) {
      const s = this._$Eu(r, n);
      s !== void 0 && this._$Eh.set(s, r);
    }
    this.elementStyles = this.finalizeStyles(this.styles);
  }
  static finalizeStyles(e) {
    const r = [];
    if (Array.isArray(e)) {
      const n = new Set(e.flat(1 / 0).reverse());
      for (const s of n) r.unshift($i(s));
    } else e !== void 0 && r.push($i(e));
    return r;
  }
  static _$Eu(e, r) {
    const n = r.attribute;
    return n === !1 ? void 0 : typeof n == "string" ? n : typeof e == "string" ? e.toLowerCase() : void 0;
  }
  constructor() {
    super(), this._$Ep = void 0, this.isUpdatePending = !1, this.hasUpdated = !1, this._$Em = null, this._$Ev();
  }
  _$Ev() {
    this._$ES = new Promise((e) => this.enableUpdating = e), this._$AL = /* @__PURE__ */ new Map(), this._$E_(), this.requestUpdate(), this.constructor.l?.forEach((e) => e(this));
  }
  addController(e) {
    (this._$EO ??= /* @__PURE__ */ new Set()).add(e), this.renderRoot !== void 0 && this.isConnected && e.hostConnected?.();
  }
  removeController(e) {
    this._$EO?.delete(e);
  }
  _$E_() {
    const e = /* @__PURE__ */ new Map(), r = this.constructor.elementProperties;
    for (const n of r.keys()) this.hasOwnProperty(n) && (e.set(n, this[n]), delete this[n]);
    e.size > 0 && (this._$Ep = e);
  }
  createRenderRoot() {
    const e = this.shadowRoot ?? this.attachShadow(this.constructor.shadowRootOptions);
    return zd(e, this.constructor.elementStyles), e;
  }
  connectedCallback() {
    this.renderRoot ??= this.createRenderRoot(), this.enableUpdating(!0), this._$EO?.forEach((e) => e.hostConnected?.());
  }
  enableUpdating(e) {
  }
  disconnectedCallback() {
    this._$EO?.forEach((e) => e.hostDisconnected?.());
  }
  attributeChangedCallback(e, r, n) {
    this._$AK(e, n);
  }
  _$ET(e, r) {
    const n = this.constructor.elementProperties.get(e), s = this.constructor._$Eu(e, n);
    if (s !== void 0 && n.reflect === !0) {
      const i = (n.converter?.toAttribute !== void 0 ? n.converter : hn).toAttribute(r, n.type);
      this._$Em = e, i == null ? this.removeAttribute(s) : this.setAttribute(s, i), this._$Em = null;
    }
  }
  _$AK(e, r) {
    const n = this.constructor, s = n._$Eh.get(e);
    if (s !== void 0 && this._$Em !== s) {
      const i = n.getPropertyOptions(s), a = typeof i.converter == "function" ? { fromAttribute: i.converter } : i.converter?.fromAttribute !== void 0 ? i.converter : hn;
      this._$Em = s;
      const l = a.fromAttribute(r, i.type);
      this[s] = l ?? this._$Ej?.get(s) ?? l, this._$Em = null;
    }
  }
  requestUpdate(e, r, n, s = !1, i) {
    if (e !== void 0) {
      const a = this.constructor;
      if (s === !1 && (i = this[e]), n ??= a.getPropertyOptions(e), !((n.hasChanged ?? Ds)(i, r) || n.useDefault && n.reflect && i === this._$Ej?.get(e) && !this.hasAttribute(a._$Eu(e, n)))) return;
      this.C(e, r, n);
    }
    this.isUpdatePending === !1 && (this._$ES = this._$EP());
  }
  C(e, r, { useDefault: n, reflect: s, wrapped: i }, a) {
    n && !(this._$Ej ??= /* @__PURE__ */ new Map()).has(e) && (this._$Ej.set(e, a ?? r ?? this[e]), i !== !0 || a !== void 0) || (this._$AL.has(e) || (this.hasUpdated || n || (r = void 0), this._$AL.set(e, r)), s === !0 && this._$Em !== e && (this._$Eq ??= /* @__PURE__ */ new Set()).add(e));
  }
  async _$EP() {
    this.isUpdatePending = !0;
    try {
      await this._$ES;
    } catch (r) {
      Promise.reject(r);
    }
    const e = this.scheduleUpdate();
    return e != null && await e, !this.isUpdatePending;
  }
  scheduleUpdate() {
    return this.performUpdate();
  }
  performUpdate() {
    if (!this.isUpdatePending) return;
    if (!this.hasUpdated) {
      if (this.renderRoot ??= this.createRenderRoot(), this._$Ep) {
        for (const [s, i] of this._$Ep) this[s] = i;
        this._$Ep = void 0;
      }
      const n = this.constructor.elementProperties;
      if (n.size > 0) for (const [s, i] of n) {
        const { wrapped: a } = i, l = this[s];
        a !== !0 || this._$AL.has(s) || l === void 0 || this.C(s, void 0, i, l);
      }
    }
    let e = !1;
    const r = this._$AL;
    try {
      e = this.shouldUpdate(r), e ? (this.willUpdate(r), this._$EO?.forEach((n) => n.hostUpdate?.()), this.update(r)) : this._$EM();
    } catch (n) {
      throw e = !1, this._$EM(), n;
    }
    e && this._$AE(r);
  }
  willUpdate(e) {
  }
  _$AE(e) {
    this._$EO?.forEach((r) => r.hostUpdated?.()), this.hasUpdated || (this.hasUpdated = !0, this.firstUpdated(e)), this.updated(e);
  }
  _$EM() {
    this._$AL = /* @__PURE__ */ new Map(), this.isUpdatePending = !1;
  }
  get updateComplete() {
    return this.getUpdateComplete();
  }
  getUpdateComplete() {
    return this._$ES;
  }
  shouldUpdate(e) {
    return !0;
  }
  update(e) {
    this._$Eq &&= this._$Eq.forEach((r) => this._$ET(r, this[r])), this._$EM();
  }
  updated(e) {
  }
  firstUpdated(e) {
  }
};
Zt.elementStyles = [], Zt.shadowRootOptions = { mode: "open" }, Zt[Er("elementProperties")] = /* @__PURE__ */ new Map(), Zt[Er("finalized")] = /* @__PURE__ */ new Map(), Yd?.({ ReactiveElement: Zt }), ($n.reactiveElementVersions ??= []).push("2.1.2");
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const Ns = globalThis, Ci = (t) => t, pn = Ns.trustedTypes, Ti = pn ? pn.createPolicy("lit-html", { createHTML: (t) => t }) : void 0, lo = "$lit$", it = `lit$${Math.random().toFixed(9).slice(2)}$`, co = "?" + it, Gd = `<${co}>`, Ot = document, Dr = () => Ot.createComment(""), Nr = (t) => t === null || typeof t != "object" && typeof t != "function", js = Array.isArray, Jd = (t) => js(t) || typeof t?.[Symbol.iterator] == "function", Wn = `[ 	
\f\r]`, pr = /<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g, Ei = /-->/g, Oi = />/g, gt = RegExp(`>|${Wn}(?:([^\\s"'>=/]+)(${Wn}*=${Wn}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`, "g"), Pi = /'/g, Di = /"/g, uo = /^(?:script|style|textarea|title)$/i, Xd = (t) => (e, ...r) => ({ _$litType$: t, strings: e, values: r }), $ = Xd(1), ht = Symbol.for("lit-noChange"), C = Symbol.for("lit-nothing"), Ni = /* @__PURE__ */ new WeakMap(), At = Ot.createTreeWalker(Ot, 129);
function fo(t, e) {
  if (!js(t) || !t.hasOwnProperty("raw")) throw Error("invalid template strings array");
  return Ti !== void 0 ? Ti.createHTML(e) : e;
}
const Qd = (t, e) => {
  const r = t.length - 1, n = [];
  let s, i = e === 2 ? "<svg>" : e === 3 ? "<math>" : "", a = pr;
  for (let l = 0; l < r; l++) {
    const c = t[l];
    let f, d, u = -1, o = 0;
    for (; o < c.length && (a.lastIndex = o, d = a.exec(c), d !== null); ) o = a.lastIndex, a === pr ? d[1] === "!--" ? a = Ei : d[1] !== void 0 ? a = Oi : d[2] !== void 0 ? (uo.test(d[2]) && (s = RegExp("</" + d[2], "g")), a = gt) : d[3] !== void 0 && (a = gt) : a === gt ? d[0] === ">" ? (a = s ?? pr, u = -1) : d[1] === void 0 ? u = -2 : (u = a.lastIndex - d[2].length, f = d[1], a = d[3] === void 0 ? gt : d[3] === '"' ? Di : Pi) : a === Di || a === Pi ? a = gt : a === Ei || a === Oi ? a = pr : (a = gt, s = void 0);
    const b = a === gt && t[l + 1].startsWith("/>") ? " " : "";
    i += a === pr ? c + Gd : u >= 0 ? (n.push(f), c.slice(0, u) + lo + c.slice(u) + it + b) : c + it + (u === -2 ? l : b);
  }
  return [fo(t, i + (t[r] || "<?>") + (e === 2 ? "</svg>" : e === 3 ? "</math>" : "")), n];
};
class jr {
  constructor({ strings: e, _$litType$: r }, n) {
    let s;
    this.parts = [];
    let i = 0, a = 0;
    const l = e.length - 1, c = this.parts, [f, d] = Qd(e, r);
    if (this.el = jr.createElement(f, n), At.currentNode = this.el.content, r === 2 || r === 3) {
      const u = this.el.content.firstChild;
      u.replaceWith(...u.childNodes);
    }
    for (; (s = At.nextNode()) !== null && c.length < l; ) {
      if (s.nodeType === 1) {
        if (s.hasAttributes()) for (const u of s.getAttributeNames()) if (u.endsWith(lo)) {
          const o = d[a++], b = s.getAttribute(u).split(it), v = /([.?@])?(.*)/.exec(o);
          c.push({ type: 1, index: i, name: v[2], strings: b, ctor: v[1] === "." ? ef : v[1] === "?" ? tf : v[1] === "@" ? rf : Sn }), s.removeAttribute(u);
        } else u.startsWith(it) && (c.push({ type: 6, index: i }), s.removeAttribute(u));
        if (uo.test(s.tagName)) {
          const u = s.textContent.split(it), o = u.length - 1;
          if (o > 0) {
            s.textContent = pn ? pn.emptyScript : "";
            for (let b = 0; b < o; b++) s.append(u[b], Dr()), At.nextNode(), c.push({ type: 2, index: ++i });
            s.append(u[o], Dr());
          }
        }
      } else if (s.nodeType === 8) if (s.data === co) c.push({ type: 2, index: i });
      else {
        let u = -1;
        for (; (u = s.data.indexOf(it, u + 1)) !== -1; ) c.push({ type: 7, index: i }), u += it.length - 1;
      }
      i++;
    }
  }
  static createElement(e, r) {
    const n = Ot.createElement("template");
    return n.innerHTML = e, n;
  }
}
function ir(t, e, r = t, n) {
  if (e === ht) return e;
  let s = n !== void 0 ? r._$Co?.[n] : r._$Cl;
  const i = Nr(e) ? void 0 : e._$litDirective$;
  return s?.constructor !== i && (s?._$AO?.(!1), i === void 0 ? s = void 0 : (s = new i(t), s._$AT(t, r, n)), n !== void 0 ? (r._$Co ??= [])[n] = s : r._$Cl = s), s !== void 0 && (e = ir(t, s._$AS(t, e.values), s, n)), e;
}
class Kd {
  constructor(e, r) {
    this._$AV = [], this._$AN = void 0, this._$AD = e, this._$AM = r;
  }
  get parentNode() {
    return this._$AM.parentNode;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  u(e) {
    const { el: { content: r }, parts: n } = this._$AD, s = (e?.creationScope ?? Ot).importNode(r, !0);
    At.currentNode = s;
    let i = At.nextNode(), a = 0, l = 0, c = n[0];
    for (; c !== void 0; ) {
      if (a === c.index) {
        let f;
        c.type === 2 ? f = new Ir(i, i.nextSibling, this, e) : c.type === 1 ? f = new c.ctor(i, c.name, c.strings, this, e) : c.type === 6 && (f = new nf(i, this, e)), this._$AV.push(f), c = n[++l];
      }
      a !== c?.index && (i = At.nextNode(), a++);
    }
    return At.currentNode = Ot, s;
  }
  p(e) {
    let r = 0;
    for (const n of this._$AV) n !== void 0 && (n.strings !== void 0 ? (n._$AI(e, n, r), r += n.strings.length - 2) : n._$AI(e[r])), r++;
  }
}
class Ir {
  get _$AU() {
    return this._$AM?._$AU ?? this._$Cv;
  }
  constructor(e, r, n, s) {
    this.type = 2, this._$AH = C, this._$AN = void 0, this._$AA = e, this._$AB = r, this._$AM = n, this.options = s, this._$Cv = s?.isConnected ?? !0;
  }
  get parentNode() {
    let e = this._$AA.parentNode;
    const r = this._$AM;
    return r !== void 0 && e?.nodeType === 11 && (e = r.parentNode), e;
  }
  get startNode() {
    return this._$AA;
  }
  get endNode() {
    return this._$AB;
  }
  _$AI(e, r = this) {
    e = ir(this, e, r), Nr(e) ? e === C || e == null || e === "" ? (this._$AH !== C && this._$AR(), this._$AH = C) : e !== this._$AH && e !== ht && this._(e) : e._$litType$ !== void 0 ? this.$(e) : e.nodeType !== void 0 ? this.T(e) : Jd(e) ? this.k(e) : this._(e);
  }
  O(e) {
    return this._$AA.parentNode.insertBefore(e, this._$AB);
  }
  T(e) {
    this._$AH !== e && (this._$AR(), this._$AH = this.O(e));
  }
  _(e) {
    this._$AH !== C && Nr(this._$AH) ? this._$AA.nextSibling.data = e : this.T(Ot.createTextNode(e)), this._$AH = e;
  }
  $(e) {
    const { values: r, _$litType$: n } = e, s = typeof n == "number" ? this._$AC(e) : (n.el === void 0 && (n.el = jr.createElement(fo(n.h, n.h[0]), this.options)), n);
    if (this._$AH?._$AD === s) this._$AH.p(r);
    else {
      const i = new Kd(s, this), a = i.u(this.options);
      i.p(r), this.T(a), this._$AH = i;
    }
  }
  _$AC(e) {
    let r = Ni.get(e.strings);
    return r === void 0 && Ni.set(e.strings, r = new jr(e)), r;
  }
  k(e) {
    js(this._$AH) || (this._$AH = [], this._$AR());
    const r = this._$AH;
    let n, s = 0;
    for (const i of e) s === r.length ? r.push(n = new Ir(this.O(Dr()), this.O(Dr()), this, this.options)) : n = r[s], n._$AI(i), s++;
    s < r.length && (this._$AR(n && n._$AB.nextSibling, s), r.length = s);
  }
  _$AR(e = this._$AA.nextSibling, r) {
    for (this._$AP?.(!1, !0, r); e !== this._$AB; ) {
      const n = Ci(e).nextSibling;
      Ci(e).remove(), e = n;
    }
  }
  setConnected(e) {
    this._$AM === void 0 && (this._$Cv = e, this._$AP?.(e));
  }
}
class Sn {
  get tagName() {
    return this.element.tagName;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  constructor(e, r, n, s, i) {
    this.type = 1, this._$AH = C, this._$AN = void 0, this.element = e, this.name = r, this._$AM = s, this.options = i, n.length > 2 || n[0] !== "" || n[1] !== "" ? (this._$AH = Array(n.length - 1).fill(new String()), this.strings = n) : this._$AH = C;
  }
  _$AI(e, r = this, n, s) {
    const i = this.strings;
    let a = !1;
    if (i === void 0) e = ir(this, e, r, 0), a = !Nr(e) || e !== this._$AH && e !== ht, a && (this._$AH = e);
    else {
      const l = e;
      let c, f;
      for (e = i[0], c = 0; c < i.length - 1; c++) f = ir(this, l[n + c], r, c), f === ht && (f = this._$AH[c]), a ||= !Nr(f) || f !== this._$AH[c], f === C ? e = C : e !== C && (e += (f ?? "") + i[c + 1]), this._$AH[c] = f;
    }
    a && !s && this.j(e);
  }
  j(e) {
    e === C ? this.element.removeAttribute(this.name) : this.element.setAttribute(this.name, e ?? "");
  }
}
class ef extends Sn {
  constructor() {
    super(...arguments), this.type = 3;
  }
  j(e) {
    this.element[this.name] = e === C ? void 0 : e;
  }
}
class tf extends Sn {
  constructor() {
    super(...arguments), this.type = 4;
  }
  j(e) {
    this.element.toggleAttribute(this.name, !!e && e !== C);
  }
}
class rf extends Sn {
  constructor(e, r, n, s, i) {
    super(e, r, n, s, i), this.type = 5;
  }
  _$AI(e, r = this) {
    if ((e = ir(this, e, r, 0) ?? C) === ht) return;
    const n = this._$AH, s = e === C && n !== C || e.capture !== n.capture || e.once !== n.once || e.passive !== n.passive, i = e !== C && (n === C || s);
    s && this.element.removeEventListener(this.name, this, n), i && this.element.addEventListener(this.name, this, e), this._$AH = e;
  }
  handleEvent(e) {
    typeof this._$AH == "function" ? this._$AH.call(this.options?.host ?? this.element, e) : this._$AH.handleEvent(e);
  }
}
class nf {
  constructor(e, r, n) {
    this.element = e, this.type = 6, this._$AN = void 0, this._$AM = r, this.options = n;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AI(e) {
    ir(this, e);
  }
}
const sf = Ns.litHtmlPolyfillSupport;
sf?.(jr, Ir), (Ns.litHtmlVersions ??= []).push("3.3.3");
const af = (t, e, r) => {
  const n = r?.renderBefore ?? e;
  let s = n._$litPart$;
  if (s === void 0) {
    const i = r?.renderBefore ?? null;
    n._$litPart$ = s = new Ir(e.insertBefore(Dr(), i), i, void 0, r ?? {});
  }
  return s._$AI(t), s;
};
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const Rs = globalThis;
let Yt = class extends Zt {
  constructor() {
    super(...arguments), this.renderOptions = { host: this }, this._$Do = void 0;
  }
  createRenderRoot() {
    const e = super.createRenderRoot();
    return this.renderOptions.renderBefore ??= e.firstChild, e;
  }
  update(e) {
    const r = this.render();
    this.hasUpdated || (this.renderOptions.isConnected = this.isConnected), super.update(e), this._$Do = af(r, this.renderRoot, this.renderOptions);
  }
  connectedCallback() {
    super.connectedCallback(), this._$Do?.setConnected(!0);
  }
  disconnectedCallback() {
    super.disconnectedCallback(), this._$Do?.setConnected(!1);
  }
  render() {
    return ht;
  }
};
Yt._$litElement$ = !0, Yt.finalized = !0, Rs.litElementHydrateSupport?.({ LitElement: Yt });
const of = Rs.litElementPolyfillSupport;
of?.({ LitElement: Yt });
(Rs.litElementVersions ??= []).push("4.2.2");
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const G = (t) => (e, r) => {
  r !== void 0 ? r.addInitializer(() => {
    customElements.define(t, e);
  }) : customElements.define(t, e);
};
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const lf = { attribute: !0, type: String, converter: hn, reflect: !1, hasChanged: Ds }, cf = (t = lf, e, r) => {
  const { kind: n, metadata: s } = r;
  let i = globalThis.litPropertyMetadata.get(s);
  if (i === void 0 && globalThis.litPropertyMetadata.set(s, i = /* @__PURE__ */ new Map()), n === "setter" && ((t = Object.create(t)).wrapped = !0), i.set(r.name, t), n === "accessor") {
    const { name: a } = r;
    return { set(l) {
      const c = e.get.call(this);
      e.set.call(this, l), this.requestUpdate(a, c, t, !0, l);
    }, init(l) {
      return l !== void 0 && this.C(a, void 0, t, l), l;
    } };
  }
  if (n === "setter") {
    const { name: a } = r;
    return function(l) {
      const c = this[a];
      e.call(this, l), this.requestUpdate(a, c, t, !0, l);
    };
  }
  throw Error("Unsupported decorator location: " + n);
};
function Ls(t) {
  return (e, r) => typeof r == "object" ? cf(t, e, r) : ((n, s, i) => {
    const a = s.hasOwnProperty(i);
    return s.constructor.createProperty(i, n), a ? Object.getOwnPropertyDescriptor(s, i) : void 0;
  })(t, e, r);
}
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
function ho(t) {
  return Ls({ ...t, state: !0, attribute: !1 });
}
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const uf = (t, e, r) => (r.configurable = !0, r.enumerable = !0, Reflect.decorate && typeof e != "object" && Object.defineProperty(t, e, r), r);
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
function df(t, e) {
  return (r, n, s) => {
    const i = (a) => a.renderRoot?.querySelector(t) ?? null;
    return uf(r, n, { get() {
      return i(this);
    } });
  };
}
/**
 * @license
 * Copyright 2020 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const po = Symbol.for(""), ff = (t) => {
  if (t?.r === po) return t?._$litStatic$;
}, mo = (t) => ({ _$litStatic$: t, r: po }), ji = /* @__PURE__ */ new Map(), hf = (t) => (e, ...r) => {
  const n = r.length;
  let s, i;
  const a = [], l = [];
  let c, f = 0, d = !1;
  for (; f < n; ) {
    for (c = e[f]; f < n && (i = r[f], (s = ff(i)) !== void 0); ) c += s + e[++f], d = !0;
    f !== n && l.push(i), a.push(c), f++;
  }
  if (f === n && a.push(e[n]), d) {
    const u = a.join("$$lit$$");
    (e = ji.get(u)) === void 0 && (a.raw = a, ji.set(u, e = a)), r = l;
  }
  return t(e, ...r);
}, go = hf($);
function vo(t, e) {
  const r = t.componentModel.type, n = e.components.get(r);
  if (!n)
    return console.warn(`Component implementation not found for type: ${r}`), C;
  const s = mo(n.tagName);
  return go`<${s} .context=${t}></${s}>`;
}
var Bn = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, mr = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-surface")], e, r = [], n, s = Yt, i, a = [], l = [], c, f = [], d = [];
  return class extends s {
    static {
      n = this;
    }
    constructor() {
      super(...arguments), this.#e = mr(this, a, void 0), this.#t = (mr(this, l), mr(this, f, !1)), this.unsubscribe = mr(this, d);
    }
    static {
      const u = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      i = [Ls({ type: Object })], c = [ho()], Bn(this, null, i, { kind: "accessor", name: "surface", static: !1, private: !1, access: { has: (o) => "surface" in o, get: (o) => o.surface, set: (o, b) => {
        o.surface = b;
      } }, metadata: u }, a, l), Bn(this, null, c, { kind: "accessor", name: "_hasRoot", static: !1, private: !1, access: { has: (o) => "_hasRoot" in o, get: (o) => o._hasRoot, set: (o, b) => {
        o._hasRoot = b;
      } }, metadata: u }, f, d), Bn(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: u }, null, r), n = e.value, u && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: u }), mr(n, r);
    }
    #e;
    /**
     * The surface model containing the component tree and catalog.
     */
    get surface() {
      return this.#e;
    }
    set surface(u) {
      this.#e = u;
    }
    #t;
    /**
     * Internal state indicating whether the root component exists.
     * @internal
     */
    get _hasRoot() {
      return this.#t;
    }
    set _hasRoot(u) {
      this.#t = u;
    }
    /**
     * Handles lifecycle updates, specifically when the `surface` property changes.
     *
     * It manages subscriptions to the components model to detect when the 'root'
     * component is created.
     *
     * @param changedProperties Map of changed properties.
     */
    willUpdate(u) {
      if (u.has("surface") && (this.unsubscribe && (this.unsubscribe(), this.unsubscribe = void 0), this._hasRoot = !!this.surface?.componentsModel.get("root"), this.surface && !this._hasRoot)) {
        const o = this.surface.componentsModel.onCreated.subscribe((b) => {
          b.id === "root" && (this._hasRoot = !0, this.unsubscribe?.(), this.unsubscribe = void 0);
        });
        this.unsubscribe = () => o.unsubscribe();
      }
    }
    /**
     * Cleans up subscriptions.
     */
    disconnectedCallback() {
      super.disconnectedCallback(), this.unsubscribe && (this.unsubscribe(), this.unsubscribe = void 0);
    }
    /**
     * Renders the surface.
     *
     * If `surface` is not set, returns `nothing`.
     * If the root component is not yet available, renders a loading state.
     * Otherwise, renders the root component using `renderA2uiNode`.
     */
    render() {
      if (!this.surface)
        return C;
      if (!this._hasRoot)
        return $`<slot name="loading"><div>Loading surface...</div></slot>`;
      try {
        const u = new no(this.surface, "root", "/");
        return $`${vo(u, this.surface.catalog)}`;
      } catch (u) {
        return console.error("Error creating root context:", u), $`<div>Error rendering surface</div>`;
      }
    }
  }, n;
})();
var pf = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Ri = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
let J = (() => {
  let t = Yt, e, r = [], n = [];
  return class extends t {
    constructor() {
      super(...arguments), this.#e = Ri(this, r, void 0), this.controller = Ri(this, n);
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(t[Symbol.metadata] ?? null) : void 0;
      e = [Ls({ type: Object })], pf(this, null, e, { kind: "accessor", name: "context", static: !1, private: !1, access: { has: (a) => "context" in a, get: (a) => a.context, set: (a, l) => {
        a.context = l;
      } }, metadata: i }, r, n), i && Object.defineProperty(this, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i });
    }
    #e;
    get context() {
      return this.#e;
    }
    set context(i) {
      this.#e = i;
    }
    /**
     * Helper method to render a child A2UI node.
     * Abstracts away the need to manually create a ComponentContext.
     *
     * @param childRef The reference to the child component to render. Can be a string ID,
     *                 a reference object containing `{ id, basePath }`, or a full inline component definition.
     * @param customPath An explicit data model path to bind the child to. If provided,
     *                   this completely overrides any path defined in the `childRef` object.
     *                   If omitted, it falls back to the `childRef`'s `basePath`, or the current component's path.
     *
     * @returns A Lit template result containing the rendered child component, or `nothing` if the reference is empty.
     */
    renderNode(i, a) {
      if (!i)
        return C;
      let l = i;
      const { surface: c, path: f } = this.context.dataContext;
      let d = a;
      return typeof i == "object" && i !== null && i.id && !i.type && (l = i.id, d = d ?? i.basePath), d = d ?? f, vo(new no(c, l, d), c.catalog);
    }
    /**
     * Reacts to changes in the component's properties.
     *
     * Specifically, when the `context` property changes or is initialized, this method
     * cleans up any existing controller and invokes `createController()` to bind to
     * the new context.
     */
    willUpdate(i) {
      super.willUpdate(i), i.has("context") && this.context && (this.controller && (this.removeController(this.controller), this.controller.dispose()), this.controller = this.createController());
    }
  };
})();
class Ms {
  static {
    this.MAX_DEPTH = 10;
  }
  /**
   * Parses an input string into an array of DynamicValues.
   * If the input contains no interpolation, it returns the raw string as a single literal.
   */
  parse(e, r = 0) {
    if (r > Ms.MAX_DEPTH)
      throw new Me("Max recursion depth reached in parse");
    if (!e || !e.includes("${"))
      return [e];
    const n = [], s = new Li(e);
    for (; !s.isAtEnd(); )
      if (s.matches("${")) {
        s.advance(2);
        const i = this.extractInterpolationContent(s), a = this.parseExpression(i, r + 1);
        a !== null && n.push(a);
      } else if (s.peek() === "\\" && s.peek(1) === "$" && s.peek(2) === "{")
        s.advance(), n.push("${"), s.advance(2);
      else {
        const i = s.pos;
        for (; !s.isAtEnd() && !(s.matches("${") || s.peek() === "\\" && s.peek(1) === "$" && s.peek(2) === "{"); )
          s.advance();
        n.push(s.input.substring(i, s.pos));
      }
    return n.filter((i) => i !== null && i !== "");
  }
  extractInterpolationContent(e) {
    const r = e.pos;
    let n = 1;
    for (; !e.isAtEnd() && n > 0; ) {
      const s = e.advance();
      if (s === "{")
        n++;
      else if (s === "}")
        n--;
      else if (s === "'" || s === '"') {
        const i = s;
        for (; !e.isAtEnd(); ) {
          const a = e.advance();
          if (a === "\\")
            e.advance();
          else if (a === i)
            break;
        }
      }
    }
    if (n > 0)
      throw new Me("Unclosed interpolation: missing '}'");
    return e.input.substring(r, e.pos - 1);
  }
  /**
   * Parses a single expression string into a DynamicValue.
   *
   * Unlike `parse()`, which handles mixed literal text and interpolations,
   * this assumes the entire string is a single expression (e.g., as found inside `${...}`).
   *
   * @param expr The expression string to parse.
   * @param depth The current recursion depth.
   * @returns The resolved DynamicValue.
   */
  parseExpression(e, r = 0) {
    if (e = e.trim(), !e)
      return "";
    const n = new Li(e), s = this.parseExpressionInternal(n, r);
    if (!n.isAtEnd())
      throw new Me(`Unexpected characters at end of expression: '${n.input.substring(n.pos)}'`);
    return s;
  }
  parseExpressionInternal(e, r) {
    if (e.skipWhitespace(), e.isAtEnd())
      return "";
    if (e.matches("${")) {
      e.advance(2);
      const s = this.extractInterpolationContent(e);
      return this.parseExpression(s, r + 1);
    }
    if (e.matchesString("'") || e.matchesString('"'))
      return this.parseStringLiteral(e);
    if (this.isDigit(e.peek()))
      return this.parseNumberLiteral(e);
    if (e.matchesKeyword("true"))
      return !0;
    if (e.matchesKeyword("false"))
      return !1;
    if (e.matchesKeyword("null"))
      return "";
    const n = this.scanPathOrIdentifier(e);
    return e.skipWhitespace(), e.peek() === "(" ? this.parseFunctionCall(n, e, r) : n ? { path: n } : "";
  }
  scanPathOrIdentifier(e) {
    const r = e.pos;
    for (; !e.isAtEnd(); ) {
      const n = e.peek();
      if (this.isAlnum(n) || n === "/" || n === "." || n === "_" || n === "-")
        e.advance();
      else
        break;
    }
    return e.input.substring(r, e.pos);
  }
  parseFunctionCall(e, r, n) {
    r.match("("), r.skipWhitespace();
    const s = {};
    for (; !r.isAtEnd() && r.peek() !== ")"; ) {
      const i = this.scanIdentifier(r);
      if (r.skipWhitespace(), !r.match(":"))
        throw new Me(`Expected ':' after argument name '${i}' in function '${e}'`);
      r.skipWhitespace(), s[i] = this.parseExpressionInternal(r, n), r.skipWhitespace(), r.peek() === "," && (r.advance(), r.skipWhitespace());
    }
    if (!r.match(")"))
      throw new Me(`Expected ')' after function arguments for '${e}'`);
    return { call: e, args: s, returnType: "any" };
  }
  scanIdentifier(e) {
    const r = e.pos;
    for (; !e.isAtEnd() && (this.isAlnum(e.peek()) || e.peek() === "_"); )
      e.advance();
    return e.input.substring(r, e.pos);
  }
  parseStringLiteral(e) {
    const r = e.advance();
    let n = "";
    for (; !e.isAtEnd(); ) {
      const s = e.advance();
      if (s === "\\") {
        const i = e.advance();
        i === "n" ? n += `
` : i === "t" ? n += "	" : i === "r" ? n += "\r" : n += i;
      } else {
        if (s === r)
          break;
        n += s;
      }
    }
    return n;
  }
  parseNumberLiteral(e) {
    const r = e.pos;
    for (; !e.isAtEnd() && (this.isDigit(e.peek()) || e.peek() === "."); )
      e.advance();
    return Number(e.input.substring(r, e.pos));
  }
  isAlnum(e) {
    return e >= "a" && e <= "z" || e >= "A" && e <= "Z" || e >= "0" && e <= "9";
  }
  isDigit(e) {
    return e >= "0" && e <= "9";
  }
}
class Li {
  constructor(e) {
    this.input = e, this.pos = 0;
  }
  isAtEnd() {
    return this.pos >= this.input.length;
  }
  peek(e = 0) {
    return this.pos + e >= this.input.length ? "\0" : this.input[this.pos + e];
  }
  advance(e = 1) {
    const r = this.input.substring(this.pos, this.pos + e);
    return this.pos += e, r;
  }
  match(e) {
    return this.peek() === e ? (this.advance(), !0) : !1;
  }
  matches(e) {
    return !!this.input.startsWith(e, this.pos);
  }
  matchesString(e) {
    return this.peek() === e;
  }
  matchesKeyword(e) {
    if (this.input.startsWith(e, this.pos)) {
      const r = this.peek(e.length);
      if (!/[a-zA-Z0-9_]/.test(r))
        return this.advance(e.length), !0;
    }
    return !1;
  }
  skipWhitespace() {
    for (; !this.isAtEnd() && /\s/.test(this.peek()); )
      this.advance();
  }
}
const bo = 6048e5, mf = 864e5, Mi = Symbol.for("constructDateFrom");
function pt(t, e) {
  return typeof t == "function" ? t(e) : t && typeof t == "object" && Mi in t ? t[Mi](e) : t instanceof Date ? new t.constructor(e) : new Date(e);
}
function Le(t, e) {
  return pt(e || t, t);
}
let gf = {};
function xn() {
  return gf;
}
function Rr(t, e) {
  const r = xn(), n = e?.weekStartsOn ?? e?.locale?.options?.weekStartsOn ?? r.weekStartsOn ?? r.locale?.options?.weekStartsOn ?? 0, s = Le(t, e?.in), i = s.getDay(), a = (i < n ? 7 : 0) + i - n;
  return s.setDate(s.getDate() - a), s.setHours(0, 0, 0, 0), s;
}
function mn(t, e) {
  return Rr(t, { ...e, weekStartsOn: 1 });
}
function yo(t, e) {
  const r = Le(t, e?.in), n = r.getFullYear(), s = pt(r, 0);
  s.setFullYear(n + 1, 0, 4), s.setHours(0, 0, 0, 0);
  const i = mn(s), a = pt(r, 0);
  a.setFullYear(n, 0, 4), a.setHours(0, 0, 0, 0);
  const l = mn(a);
  return r.getTime() >= i.getTime() ? n + 1 : r.getTime() >= l.getTime() ? n : n - 1;
}
function Fi(t) {
  const e = Le(t), r = new Date(
    Date.UTC(
      e.getFullYear(),
      e.getMonth(),
      e.getDate(),
      e.getHours(),
      e.getMinutes(),
      e.getSeconds(),
      e.getMilliseconds()
    )
  );
  return r.setUTCFullYear(e.getFullYear()), +t - +r;
}
function vf(t, ...e) {
  const r = pt.bind(
    null,
    e.find((n) => typeof n == "object")
  );
  return e.map(r);
}
function Ii(t, e) {
  const r = Le(t, e?.in);
  return r.setHours(0, 0, 0, 0), r;
}
function bf(t, e, r) {
  const [n, s] = vf(
    r?.in,
    t,
    e
  ), i = Ii(n), a = Ii(s), l = +i - Fi(i), c = +a - Fi(a);
  return Math.round((l - c) / mf);
}
function yf(t, e) {
  const r = yo(t, e), n = pt(t, 0);
  return n.setFullYear(r, 0, 4), n.setHours(0, 0, 0, 0), mn(n);
}
function _f(t) {
  return t instanceof Date || typeof t == "object" && Object.prototype.toString.call(t) === "[object Date]";
}
function Af(t) {
  return !(!_f(t) && typeof t != "number" || isNaN(+Le(t)));
}
function wf(t, e) {
  const r = Le(t, e?.in);
  return r.setFullYear(r.getFullYear(), 0, 1), r.setHours(0, 0, 0, 0), r;
}
const kf = {
  lessThanXSeconds: {
    one: "less than a second",
    other: "less than {{count}} seconds"
  },
  xSeconds: {
    one: "1 second",
    other: "{{count}} seconds"
  },
  halfAMinute: "half a minute",
  lessThanXMinutes: {
    one: "less than a minute",
    other: "less than {{count}} minutes"
  },
  xMinutes: {
    one: "1 minute",
    other: "{{count}} minutes"
  },
  aboutXHours: {
    one: "about 1 hour",
    other: "about {{count}} hours"
  },
  xHours: {
    one: "1 hour",
    other: "{{count}} hours"
  },
  xDays: {
    one: "1 day",
    other: "{{count}} days"
  },
  aboutXWeeks: {
    one: "about 1 week",
    other: "about {{count}} weeks"
  },
  xWeeks: {
    one: "1 week",
    other: "{{count}} weeks"
  },
  aboutXMonths: {
    one: "about 1 month",
    other: "about {{count}} months"
  },
  xMonths: {
    one: "1 month",
    other: "{{count}} months"
  },
  aboutXYears: {
    one: "about 1 year",
    other: "about {{count}} years"
  },
  xYears: {
    one: "1 year",
    other: "{{count}} years"
  },
  overXYears: {
    one: "over 1 year",
    other: "over {{count}} years"
  },
  almostXYears: {
    one: "almost 1 year",
    other: "almost {{count}} years"
  }
}, $f = (t, e, r) => {
  let n;
  const s = kf[t];
  return typeof s == "string" ? n = s : e === 1 ? n = s.one : n = s.other.replace("{{count}}", e.toString()), r?.addSuffix ? r.comparison && r.comparison > 0 ? "in " + n : n + " ago" : n;
};
function qn(t) {
  return (e = {}) => {
    const r = e.width ? String(e.width) : t.defaultWidth;
    return t.formats[r] || t.formats[t.defaultWidth];
  };
}
const Sf = {
  full: "EEEE, MMMM do, y",
  long: "MMMM do, y",
  medium: "MMM d, y",
  short: "MM/dd/yyyy"
}, xf = {
  full: "h:mm:ss a zzzz",
  long: "h:mm:ss a z",
  medium: "h:mm:ss a",
  short: "h:mm a"
}, Cf = {
  full: "{{date}} 'at' {{time}}",
  long: "{{date}} 'at' {{time}}",
  medium: "{{date}}, {{time}}",
  short: "{{date}}, {{time}}"
}, Tf = {
  date: qn({
    formats: Sf,
    defaultWidth: "full"
  }),
  time: qn({
    formats: xf,
    defaultWidth: "full"
  }),
  dateTime: qn({
    formats: Cf,
    defaultWidth: "full"
  })
}, Ef = {
  lastWeek: "'last' eeee 'at' p",
  yesterday: "'yesterday at' p",
  today: "'today at' p",
  tomorrow: "'tomorrow at' p",
  nextWeek: "eeee 'at' p",
  other: "P"
}, Of = (t, e, r, n) => Ef[t];
function gr(t) {
  return (e, r) => {
    const n = r?.context ? String(r.context) : "standalone";
    let s;
    if (n === "formatting" && t.formattingValues) {
      const a = t.defaultFormattingWidth || t.defaultWidth, l = r?.width ? String(r.width) : a;
      s = t.formattingValues[l] || t.formattingValues[a];
    } else {
      const a = t.defaultWidth, l = r?.width ? String(r.width) : t.defaultWidth;
      s = t.values[l] || t.values[a];
    }
    const i = t.argumentCallback ? t.argumentCallback(e) : e;
    return s[i];
  };
}
const Pf = {
  narrow: ["B", "A"],
  abbreviated: ["BC", "AD"],
  wide: ["Before Christ", "Anno Domini"]
}, Df = {
  narrow: ["1", "2", "3", "4"],
  abbreviated: ["Q1", "Q2", "Q3", "Q4"],
  wide: ["1st quarter", "2nd quarter", "3rd quarter", "4th quarter"]
}, Nf = {
  narrow: ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"],
  abbreviated: [
    "Jan",
    "Feb",
    "Mar",
    "Apr",
    "May",
    "Jun",
    "Jul",
    "Aug",
    "Sep",
    "Oct",
    "Nov",
    "Dec"
  ],
  wide: [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December"
  ]
}, jf = {
  narrow: ["S", "M", "T", "W", "T", "F", "S"],
  short: ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"],
  abbreviated: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
  wide: [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday"
  ]
}, Rf = {
  narrow: {
    am: "a",
    pm: "p",
    midnight: "mi",
    noon: "n",
    morning: "morning",
    afternoon: "afternoon",
    evening: "evening",
    night: "night"
  },
  abbreviated: {
    am: "AM",
    pm: "PM",
    midnight: "midnight",
    noon: "noon",
    morning: "morning",
    afternoon: "afternoon",
    evening: "evening",
    night: "night"
  },
  wide: {
    am: "a.m.",
    pm: "p.m.",
    midnight: "midnight",
    noon: "noon",
    morning: "morning",
    afternoon: "afternoon",
    evening: "evening",
    night: "night"
  }
}, Lf = {
  narrow: {
    am: "a",
    pm: "p",
    midnight: "mi",
    noon: "n",
    morning: "in the morning",
    afternoon: "in the afternoon",
    evening: "in the evening",
    night: "at night"
  },
  abbreviated: {
    am: "AM",
    pm: "PM",
    midnight: "midnight",
    noon: "noon",
    morning: "in the morning",
    afternoon: "in the afternoon",
    evening: "in the evening",
    night: "at night"
  },
  wide: {
    am: "a.m.",
    pm: "p.m.",
    midnight: "midnight",
    noon: "noon",
    morning: "in the morning",
    afternoon: "in the afternoon",
    evening: "in the evening",
    night: "at night"
  }
}, Mf = (t, e) => {
  const r = Number(t), n = r % 100;
  if (n > 20 || n < 10)
    switch (n % 10) {
      case 1:
        return r + "st";
      case 2:
        return r + "nd";
      case 3:
        return r + "rd";
    }
  return r + "th";
}, Ff = {
  ordinalNumber: Mf,
  era: gr({
    values: Pf,
    defaultWidth: "wide"
  }),
  quarter: gr({
    values: Df,
    defaultWidth: "wide",
    argumentCallback: (t) => t - 1
  }),
  month: gr({
    values: Nf,
    defaultWidth: "wide"
  }),
  day: gr({
    values: jf,
    defaultWidth: "wide"
  }),
  dayPeriod: gr({
    values: Rf,
    defaultWidth: "wide",
    formattingValues: Lf,
    defaultFormattingWidth: "wide"
  })
};
function vr(t) {
  return (e, r = {}) => {
    const n = r.width, s = n && t.matchPatterns[n] || t.matchPatterns[t.defaultMatchWidth], i = e.match(s);
    if (!i)
      return null;
    const a = i[0], l = n && t.parsePatterns[n] || t.parsePatterns[t.defaultParseWidth], c = Array.isArray(l) ? zf(l, (u) => u.test(a)) : (
      // [TODO] -- I challenge you to fix the type
      If(l, (u) => u.test(a))
    );
    let f;
    f = t.valueCallback ? t.valueCallback(c) : c, f = r.valueCallback ? (
      // [TODO] -- I challenge you to fix the type
      r.valueCallback(f)
    ) : f;
    const d = e.slice(a.length);
    return { value: f, rest: d };
  };
}
function If(t, e) {
  for (const r in t)
    if (Object.prototype.hasOwnProperty.call(t, r) && e(t[r]))
      return r;
}
function zf(t, e) {
  for (let r = 0; r < t.length; r++)
    if (e(t[r]))
      return r;
}
function Zf(t) {
  return (e, r = {}) => {
    const n = e.match(t.matchPattern);
    if (!n) return null;
    const s = n[0], i = e.match(t.parsePattern);
    if (!i) return null;
    let a = t.valueCallback ? t.valueCallback(i[0]) : i[0];
    a = r.valueCallback ? r.valueCallback(a) : a;
    const l = e.slice(s.length);
    return { value: a, rest: l };
  };
}
const Uf = /^(\d+)(th|st|nd|rd)?/i, Vf = /\d+/i, Wf = {
  narrow: /^(b|a)/i,
  abbreviated: /^(b\.?\s?c\.?|b\.?\s?c\.?\s?e\.?|a\.?\s?d\.?|c\.?\s?e\.?)/i,
  wide: /^(before christ|before common era|anno domini|common era)/i
}, Bf = {
  any: [/^b/i, /^(a|c)/i]
}, qf = {
  narrow: /^[1234]/i,
  abbreviated: /^q[1234]/i,
  wide: /^[1234](th|st|nd|rd)? quarter/i
}, Hf = {
  any: [/1/i, /2/i, /3/i, /4/i]
}, Yf = {
  narrow: /^[jfmasond]/i,
  abbreviated: /^(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)/i,
  wide: /^(january|february|march|april|may|june|july|august|september|october|november|december)/i
}, Gf = {
  narrow: [
    /^j/i,
    /^f/i,
    /^m/i,
    /^a/i,
    /^m/i,
    /^j/i,
    /^j/i,
    /^a/i,
    /^s/i,
    /^o/i,
    /^n/i,
    /^d/i
  ],
  any: [
    /^ja/i,
    /^f/i,
    /^mar/i,
    /^ap/i,
    /^may/i,
    /^jun/i,
    /^jul/i,
    /^au/i,
    /^s/i,
    /^o/i,
    /^n/i,
    /^d/i
  ]
}, Jf = {
  narrow: /^[smtwf]/i,
  short: /^(su|mo|tu|we|th|fr|sa)/i,
  abbreviated: /^(sun|mon|tue|wed|thu|fri|sat)/i,
  wide: /^(sunday|monday|tuesday|wednesday|thursday|friday|saturday)/i
}, Xf = {
  narrow: [/^s/i, /^m/i, /^t/i, /^w/i, /^t/i, /^f/i, /^s/i],
  any: [/^su/i, /^m/i, /^tu/i, /^w/i, /^th/i, /^f/i, /^sa/i]
}, Qf = {
  narrow: /^(a|p|mi|n|(in the|at) (morning|afternoon|evening|night))/i,
  any: /^([ap]\.?\s?m\.?|midnight|noon|(in the|at) (morning|afternoon|evening|night))/i
}, Kf = {
  any: {
    am: /^a/i,
    pm: /^p/i,
    midnight: /^mi/i,
    noon: /^no/i,
    morning: /morning/i,
    afternoon: /afternoon/i,
    evening: /evening/i,
    night: /night/i
  }
}, eh = {
  ordinalNumber: Zf({
    matchPattern: Uf,
    parsePattern: Vf,
    valueCallback: (t) => parseInt(t, 10)
  }),
  era: vr({
    matchPatterns: Wf,
    defaultMatchWidth: "wide",
    parsePatterns: Bf,
    defaultParseWidth: "any"
  }),
  quarter: vr({
    matchPatterns: qf,
    defaultMatchWidth: "wide",
    parsePatterns: Hf,
    defaultParseWidth: "any",
    valueCallback: (t) => t + 1
  }),
  month: vr({
    matchPatterns: Yf,
    defaultMatchWidth: "wide",
    parsePatterns: Gf,
    defaultParseWidth: "any"
  }),
  day: vr({
    matchPatterns: Jf,
    defaultMatchWidth: "wide",
    parsePatterns: Xf,
    defaultParseWidth: "any"
  }),
  dayPeriod: vr({
    matchPatterns: Qf,
    defaultMatchWidth: "any",
    parsePatterns: Kf,
    defaultParseWidth: "any"
  })
}, th = {
  code: "en-US",
  formatDistance: $f,
  formatLong: Tf,
  formatRelative: Of,
  localize: Ff,
  match: eh,
  options: {
    weekStartsOn: 0,
    firstWeekContainsDate: 1
  }
};
function rh(t, e) {
  const r = Le(t, e?.in);
  return bf(r, wf(r)) + 1;
}
function nh(t, e) {
  const r = Le(t, e?.in), n = +mn(r) - +yf(r);
  return Math.round(n / bo) + 1;
}
function _o(t, e) {
  const r = Le(t, e?.in), n = r.getFullYear(), s = xn(), i = e?.firstWeekContainsDate ?? e?.locale?.options?.firstWeekContainsDate ?? s.firstWeekContainsDate ?? s.locale?.options?.firstWeekContainsDate ?? 1, a = pt(e?.in || t, 0);
  a.setFullYear(n + 1, 0, i), a.setHours(0, 0, 0, 0);
  const l = Rr(a, e), c = pt(e?.in || t, 0);
  c.setFullYear(n, 0, i), c.setHours(0, 0, 0, 0);
  const f = Rr(c, e);
  return +r >= +l ? n + 1 : +r >= +f ? n : n - 1;
}
function sh(t, e) {
  const r = xn(), n = e?.firstWeekContainsDate ?? e?.locale?.options?.firstWeekContainsDate ?? r.firstWeekContainsDate ?? r.locale?.options?.firstWeekContainsDate ?? 1, s = _o(t, e), i = pt(e?.in || t, 0);
  return i.setFullYear(s, 0, n), i.setHours(0, 0, 0, 0), Rr(i, e);
}
function ih(t, e) {
  const r = Le(t, e?.in), n = +Rr(r, e) - +sh(r, e);
  return Math.round(n / bo) + 1;
}
function V(t, e) {
  const r = t < 0 ? "-" : "", n = Math.abs(t).toString().padStart(e, "0");
  return r + n;
}
const nt = {
  // Year
  y(t, e) {
    const r = t.getFullYear(), n = r > 0 ? r : 1 - r;
    return V(e === "yy" ? n % 100 : n, e.length);
  },
  // Month
  M(t, e) {
    const r = t.getMonth();
    return e === "M" ? String(r + 1) : V(r + 1, 2);
  },
  // Day of the month
  d(t, e) {
    return V(t.getDate(), e.length);
  },
  // AM or PM
  a(t, e) {
    const r = t.getHours() / 12 >= 1 ? "pm" : "am";
    switch (e) {
      case "a":
      case "aa":
        return r.toUpperCase();
      case "aaa":
        return r;
      case "aaaaa":
        return r[0];
      case "aaaa":
      default:
        return r === "am" ? "a.m." : "p.m.";
    }
  },
  // Hour [1-12]
  h(t, e) {
    return V(t.getHours() % 12 || 12, e.length);
  },
  // Hour [0-23]
  H(t, e) {
    return V(t.getHours(), e.length);
  },
  // Minute
  m(t, e) {
    return V(t.getMinutes(), e.length);
  },
  // Second
  s(t, e) {
    return V(t.getSeconds(), e.length);
  },
  // Fraction of second
  S(t, e) {
    const r = e.length, n = t.getMilliseconds(), s = Math.trunc(
      n * Math.pow(10, r - 3)
    );
    return V(s, e.length);
  }
}, It = {
  midnight: "midnight",
  noon: "noon",
  morning: "morning",
  afternoon: "afternoon",
  evening: "evening",
  night: "night"
}, zi = {
  // Era
  G: function(t, e, r) {
    const n = t.getFullYear() > 0 ? 1 : 0;
    switch (e) {
      // AD, BC
      case "G":
      case "GG":
      case "GGG":
        return r.era(n, { width: "abbreviated" });
      // A, B
      case "GGGGG":
        return r.era(n, { width: "narrow" });
      // Anno Domini, Before Christ
      case "GGGG":
      default:
        return r.era(n, { width: "wide" });
    }
  },
  // Year
  y: function(t, e, r) {
    if (e === "yo") {
      const n = t.getFullYear(), s = n > 0 ? n : 1 - n;
      return r.ordinalNumber(s, { unit: "year" });
    }
    return nt.y(t, e);
  },
  // Local week-numbering year
  Y: function(t, e, r, n) {
    const s = _o(t, n), i = s > 0 ? s : 1 - s;
    if (e === "YY") {
      const a = i % 100;
      return V(a, 2);
    }
    return e === "Yo" ? r.ordinalNumber(i, { unit: "year" }) : V(i, e.length);
  },
  // ISO week-numbering year
  R: function(t, e) {
    const r = yo(t);
    return V(r, e.length);
  },
  // Extended year. This is a single number designating the year of this calendar system.
  // The main difference between `y` and `u` localizers are B.C. years:
  // | Year | `y` | `u` |
  // |------|-----|-----|
  // | AC 1 |   1 |   1 |
  // | BC 1 |   1 |   0 |
  // | BC 2 |   2 |  -1 |
  // Also `yy` always returns the last two digits of a year,
  // while `uu` pads single digit years to 2 characters and returns other years unchanged.
  u: function(t, e) {
    const r = t.getFullYear();
    return V(r, e.length);
  },
  // Quarter
  Q: function(t, e, r) {
    const n = Math.ceil((t.getMonth() + 1) / 3);
    switch (e) {
      // 1, 2, 3, 4
      case "Q":
        return String(n);
      // 01, 02, 03, 04
      case "QQ":
        return V(n, 2);
      // 1st, 2nd, 3rd, 4th
      case "Qo":
        return r.ordinalNumber(n, { unit: "quarter" });
      // Q1, Q2, Q3, Q4
      case "QQQ":
        return r.quarter(n, {
          width: "abbreviated",
          context: "formatting"
        });
      // 1, 2, 3, 4 (narrow quarter; could be not numerical)
      case "QQQQQ":
        return r.quarter(n, {
          width: "narrow",
          context: "formatting"
        });
      // 1st quarter, 2nd quarter, ...
      case "QQQQ":
      default:
        return r.quarter(n, {
          width: "wide",
          context: "formatting"
        });
    }
  },
  // Stand-alone quarter
  q: function(t, e, r) {
    const n = Math.ceil((t.getMonth() + 1) / 3);
    switch (e) {
      // 1, 2, 3, 4
      case "q":
        return String(n);
      // 01, 02, 03, 04
      case "qq":
        return V(n, 2);
      // 1st, 2nd, 3rd, 4th
      case "qo":
        return r.ordinalNumber(n, { unit: "quarter" });
      // Q1, Q2, Q3, Q4
      case "qqq":
        return r.quarter(n, {
          width: "abbreviated",
          context: "standalone"
        });
      // 1, 2, 3, 4 (narrow quarter; could be not numerical)
      case "qqqqq":
        return r.quarter(n, {
          width: "narrow",
          context: "standalone"
        });
      // 1st quarter, 2nd quarter, ...
      case "qqqq":
      default:
        return r.quarter(n, {
          width: "wide",
          context: "standalone"
        });
    }
  },
  // Month
  M: function(t, e, r) {
    const n = t.getMonth();
    switch (e) {
      case "M":
      case "MM":
        return nt.M(t, e);
      // 1st, 2nd, ..., 12th
      case "Mo":
        return r.ordinalNumber(n + 1, { unit: "month" });
      // Jan, Feb, ..., Dec
      case "MMM":
        return r.month(n, {
          width: "abbreviated",
          context: "formatting"
        });
      // J, F, ..., D
      case "MMMMM":
        return r.month(n, {
          width: "narrow",
          context: "formatting"
        });
      // January, February, ..., December
      case "MMMM":
      default:
        return r.month(n, { width: "wide", context: "formatting" });
    }
  },
  // Stand-alone month
  L: function(t, e, r) {
    const n = t.getMonth();
    switch (e) {
      // 1, 2, ..., 12
      case "L":
        return String(n + 1);
      // 01, 02, ..., 12
      case "LL":
        return V(n + 1, 2);
      // 1st, 2nd, ..., 12th
      case "Lo":
        return r.ordinalNumber(n + 1, { unit: "month" });
      // Jan, Feb, ..., Dec
      case "LLL":
        return r.month(n, {
          width: "abbreviated",
          context: "standalone"
        });
      // J, F, ..., D
      case "LLLLL":
        return r.month(n, {
          width: "narrow",
          context: "standalone"
        });
      // January, February, ..., December
      case "LLLL":
      default:
        return r.month(n, { width: "wide", context: "standalone" });
    }
  },
  // Local week of year
  w: function(t, e, r, n) {
    const s = ih(t, n);
    return e === "wo" ? r.ordinalNumber(s, { unit: "week" }) : V(s, e.length);
  },
  // ISO week of year
  I: function(t, e, r) {
    const n = nh(t);
    return e === "Io" ? r.ordinalNumber(n, { unit: "week" }) : V(n, e.length);
  },
  // Day of the month
  d: function(t, e, r) {
    return e === "do" ? r.ordinalNumber(t.getDate(), { unit: "date" }) : nt.d(t, e);
  },
  // Day of year
  D: function(t, e, r) {
    const n = rh(t);
    return e === "Do" ? r.ordinalNumber(n, { unit: "dayOfYear" }) : V(n, e.length);
  },
  // Day of week
  E: function(t, e, r) {
    const n = t.getDay();
    switch (e) {
      // Tue
      case "E":
      case "EE":
      case "EEE":
        return r.day(n, {
          width: "abbreviated",
          context: "formatting"
        });
      // T
      case "EEEEE":
        return r.day(n, {
          width: "narrow",
          context: "formatting"
        });
      // Tu
      case "EEEEEE":
        return r.day(n, {
          width: "short",
          context: "formatting"
        });
      // Tuesday
      case "EEEE":
      default:
        return r.day(n, {
          width: "wide",
          context: "formatting"
        });
    }
  },
  // Local day of week
  e: function(t, e, r, n) {
    const s = t.getDay(), i = (s - n.weekStartsOn + 8) % 7 || 7;
    switch (e) {
      // Numerical value (Nth day of week with current locale or weekStartsOn)
      case "e":
        return String(i);
      // Padded numerical value
      case "ee":
        return V(i, 2);
      // 1st, 2nd, ..., 7th
      case "eo":
        return r.ordinalNumber(i, { unit: "day" });
      case "eee":
        return r.day(s, {
          width: "abbreviated",
          context: "formatting"
        });
      // T
      case "eeeee":
        return r.day(s, {
          width: "narrow",
          context: "formatting"
        });
      // Tu
      case "eeeeee":
        return r.day(s, {
          width: "short",
          context: "formatting"
        });
      // Tuesday
      case "eeee":
      default:
        return r.day(s, {
          width: "wide",
          context: "formatting"
        });
    }
  },
  // Stand-alone local day of week
  c: function(t, e, r, n) {
    const s = t.getDay(), i = (s - n.weekStartsOn + 8) % 7 || 7;
    switch (e) {
      // Numerical value (same as in `e`)
      case "c":
        return String(i);
      // Padded numerical value
      case "cc":
        return V(i, e.length);
      // 1st, 2nd, ..., 7th
      case "co":
        return r.ordinalNumber(i, { unit: "day" });
      case "ccc":
        return r.day(s, {
          width: "abbreviated",
          context: "standalone"
        });
      // T
      case "ccccc":
        return r.day(s, {
          width: "narrow",
          context: "standalone"
        });
      // Tu
      case "cccccc":
        return r.day(s, {
          width: "short",
          context: "standalone"
        });
      // Tuesday
      case "cccc":
      default:
        return r.day(s, {
          width: "wide",
          context: "standalone"
        });
    }
  },
  // ISO day of week
  i: function(t, e, r) {
    const n = t.getDay(), s = n === 0 ? 7 : n;
    switch (e) {
      // 2
      case "i":
        return String(s);
      // 02
      case "ii":
        return V(s, e.length);
      // 2nd
      case "io":
        return r.ordinalNumber(s, { unit: "day" });
      // Tue
      case "iii":
        return r.day(n, {
          width: "abbreviated",
          context: "formatting"
        });
      // T
      case "iiiii":
        return r.day(n, {
          width: "narrow",
          context: "formatting"
        });
      // Tu
      case "iiiiii":
        return r.day(n, {
          width: "short",
          context: "formatting"
        });
      // Tuesday
      case "iiii":
      default:
        return r.day(n, {
          width: "wide",
          context: "formatting"
        });
    }
  },
  // AM or PM
  a: function(t, e, r) {
    const s = t.getHours() / 12 >= 1 ? "pm" : "am";
    switch (e) {
      case "a":
      case "aa":
        return r.dayPeriod(s, {
          width: "abbreviated",
          context: "formatting"
        });
      case "aaa":
        return r.dayPeriod(s, {
          width: "abbreviated",
          context: "formatting"
        }).toLowerCase();
      case "aaaaa":
        return r.dayPeriod(s, {
          width: "narrow",
          context: "formatting"
        });
      case "aaaa":
      default:
        return r.dayPeriod(s, {
          width: "wide",
          context: "formatting"
        });
    }
  },
  // AM, PM, midnight, noon
  b: function(t, e, r) {
    const n = t.getHours();
    let s;
    switch (n === 12 ? s = It.noon : n === 0 ? s = It.midnight : s = n / 12 >= 1 ? "pm" : "am", e) {
      case "b":
      case "bb":
        return r.dayPeriod(s, {
          width: "abbreviated",
          context: "formatting"
        });
      case "bbb":
        return r.dayPeriod(s, {
          width: "abbreviated",
          context: "formatting"
        }).toLowerCase();
      case "bbbbb":
        return r.dayPeriod(s, {
          width: "narrow",
          context: "formatting"
        });
      case "bbbb":
      default:
        return r.dayPeriod(s, {
          width: "wide",
          context: "formatting"
        });
    }
  },
  // in the morning, in the afternoon, in the evening, at night
  B: function(t, e, r) {
    const n = t.getHours();
    let s;
    switch (n >= 17 ? s = It.evening : n >= 12 ? s = It.afternoon : n >= 4 ? s = It.morning : s = It.night, e) {
      case "B":
      case "BB":
      case "BBB":
        return r.dayPeriod(s, {
          width: "abbreviated",
          context: "formatting"
        });
      case "BBBBB":
        return r.dayPeriod(s, {
          width: "narrow",
          context: "formatting"
        });
      case "BBBB":
      default:
        return r.dayPeriod(s, {
          width: "wide",
          context: "formatting"
        });
    }
  },
  // Hour [1-12]
  h: function(t, e, r) {
    if (e === "ho") {
      let n = t.getHours() % 12;
      return n === 0 && (n = 12), r.ordinalNumber(n, { unit: "hour" });
    }
    return nt.h(t, e);
  },
  // Hour [0-23]
  H: function(t, e, r) {
    return e === "Ho" ? r.ordinalNumber(t.getHours(), { unit: "hour" }) : nt.H(t, e);
  },
  // Hour [0-11]
  K: function(t, e, r) {
    const n = t.getHours() % 12;
    return e === "Ko" ? r.ordinalNumber(n, { unit: "hour" }) : V(n, e.length);
  },
  // Hour [1-24]
  k: function(t, e, r) {
    let n = t.getHours();
    return n === 0 && (n = 24), e === "ko" ? r.ordinalNumber(n, { unit: "hour" }) : V(n, e.length);
  },
  // Minute
  m: function(t, e, r) {
    return e === "mo" ? r.ordinalNumber(t.getMinutes(), { unit: "minute" }) : nt.m(t, e);
  },
  // Second
  s: function(t, e, r) {
    return e === "so" ? r.ordinalNumber(t.getSeconds(), { unit: "second" }) : nt.s(t, e);
  },
  // Fraction of second
  S: function(t, e) {
    return nt.S(t, e);
  },
  // Timezone (ISO-8601. If offset is 0, output is always `'Z'`)
  X: function(t, e, r) {
    const n = t.getTimezoneOffset();
    if (n === 0)
      return "Z";
    switch (e) {
      // Hours and optional minutes
      case "X":
        return Ui(n);
      // Hours, minutes and optional seconds without `:` delimiter
      // Note: neither ISO-8601 nor JavaScript supports seconds in timezone offsets
      // so this token always has the same output as `XX`
      case "XXXX":
      case "XX":
        return bt(n);
      // Hours, minutes and optional seconds with `:` delimiter
      // Note: neither ISO-8601 nor JavaScript supports seconds in timezone offsets
      // so this token always has the same output as `XXX`
      case "XXXXX":
      case "XXX":
      // Hours and minutes with `:` delimiter
      default:
        return bt(n, ":");
    }
  },
  // Timezone (ISO-8601. If offset is 0, output is `'+00:00'` or equivalent)
  x: function(t, e, r) {
    const n = t.getTimezoneOffset();
    switch (e) {
      // Hours and optional minutes
      case "x":
        return Ui(n);
      // Hours, minutes and optional seconds without `:` delimiter
      // Note: neither ISO-8601 nor JavaScript supports seconds in timezone offsets
      // so this token always has the same output as `xx`
      case "xxxx":
      case "xx":
        return bt(n);
      // Hours, minutes and optional seconds with `:` delimiter
      // Note: neither ISO-8601 nor JavaScript supports seconds in timezone offsets
      // so this token always has the same output as `xxx`
      case "xxxxx":
      case "xxx":
      // Hours and minutes with `:` delimiter
      default:
        return bt(n, ":");
    }
  },
  // Timezone (GMT)
  O: function(t, e, r) {
    const n = t.getTimezoneOffset();
    switch (e) {
      // Short
      case "O":
      case "OO":
      case "OOO":
        return "GMT" + Zi(n, ":");
      // Long
      case "OOOO":
      default:
        return "GMT" + bt(n, ":");
    }
  },
  // Timezone (specific non-location)
  z: function(t, e, r) {
    const n = t.getTimezoneOffset();
    switch (e) {
      // Short
      case "z":
      case "zz":
      case "zzz":
        return "GMT" + Zi(n, ":");
      // Long
      case "zzzz":
      default:
        return "GMT" + bt(n, ":");
    }
  },
  // Seconds timestamp
  t: function(t, e, r) {
    const n = Math.trunc(+t / 1e3);
    return V(n, e.length);
  },
  // Milliseconds timestamp
  T: function(t, e, r) {
    return V(+t, e.length);
  }
};
function Zi(t, e = "") {
  const r = t > 0 ? "-" : "+", n = Math.abs(t), s = Math.trunc(n / 60), i = n % 60;
  return i === 0 ? r + String(s) : r + String(s) + e + V(i, 2);
}
function Ui(t, e) {
  return t % 60 === 0 ? (t > 0 ? "-" : "+") + V(Math.abs(t) / 60, 2) : bt(t, e);
}
function bt(t, e = "") {
  const r = t > 0 ? "-" : "+", n = Math.abs(t), s = V(Math.trunc(n / 60), 2), i = V(n % 60, 2);
  return r + s + e + i;
}
const Vi = (t, e) => {
  switch (t) {
    case "P":
      return e.date({ width: "short" });
    case "PP":
      return e.date({ width: "medium" });
    case "PPP":
      return e.date({ width: "long" });
    case "PPPP":
    default:
      return e.date({ width: "full" });
  }
}, Ao = (t, e) => {
  switch (t) {
    case "p":
      return e.time({ width: "short" });
    case "pp":
      return e.time({ width: "medium" });
    case "ppp":
      return e.time({ width: "long" });
    case "pppp":
    default:
      return e.time({ width: "full" });
  }
}, ah = (t, e) => {
  const r = t.match(/(P+)(p+)?/) || [], n = r[1], s = r[2];
  if (!s)
    return Vi(t, e);
  let i;
  switch (n) {
    case "P":
      i = e.dateTime({ width: "short" });
      break;
    case "PP":
      i = e.dateTime({ width: "medium" });
      break;
    case "PPP":
      i = e.dateTime({ width: "long" });
      break;
    case "PPPP":
    default:
      i = e.dateTime({ width: "full" });
      break;
  }
  return i.replace("{{date}}", Vi(n, e)).replace("{{time}}", Ao(s, e));
}, oh = {
  p: Ao,
  P: ah
}, lh = /^D+$/, ch = /^Y+$/, uh = ["D", "DD", "YY", "YYYY"];
function dh(t) {
  return lh.test(t);
}
function fh(t) {
  return ch.test(t);
}
function hh(t, e, r) {
  const n = ph(t, e, r);
  if (console.warn(n), uh.includes(t)) throw new RangeError(n);
}
function ph(t, e, r) {
  const n = t[0] === "Y" ? "years" : "days of the month";
  return `Use \`${t.toLowerCase()}\` instead of \`${t}\` (in \`${e}\`) for formatting ${n} to the input \`${r}\`; see: https://github.com/date-fns/date-fns/blob/master/docs/unicodeTokens.md`;
}
const mh = /[yYQqMLwIdDecihHKkms]o|(\w)\1*|''|'(''|[^'])+('|$)|./g, gh = /P+p+|P+|p+|''|'(''|[^'])+('|$)|./g, vh = /^'([^]*?)'?$/, bh = /''/g, yh = /[a-zA-Z]/;
function _h(t, e, r) {
  const n = xn(), s = n.locale ?? th, i = n.firstWeekContainsDate ?? n.locale?.options?.firstWeekContainsDate ?? 1, a = n.weekStartsOn ?? n.locale?.options?.weekStartsOn ?? 0, l = Le(t, r?.in);
  if (!Af(l))
    throw new RangeError("Invalid time value");
  let c = e.match(gh).map((d) => {
    const u = d[0];
    if (u === "p" || u === "P") {
      const o = oh[u];
      return o(d, s.formatLong);
    }
    return d;
  }).join("").match(mh).map((d) => {
    if (d === "''")
      return { isToken: !1, value: "'" };
    const u = d[0];
    if (u === "'")
      return { isToken: !1, value: Ah(d) };
    if (zi[u])
      return { isToken: !0, value: d };
    if (u.match(yh))
      throw new RangeError(
        "Format string contains an unescaped latin alphabet character `" + u + "`"
      );
    return { isToken: !1, value: d };
  });
  s.localize.preprocessor && (c = s.localize.preprocessor(l, c));
  const f = {
    firstWeekContainsDate: i,
    weekStartsOn: a,
    locale: s
  };
  return c.map((d) => {
    if (!d.isToken) return d.value;
    const u = d.value;
    (fh(u) || dh(u)) && hh(u, e, String(t));
    const o = zi[u[0]];
    return o(l, u, s.localize, f);
  }).join("");
}
function Ah(t) {
  const e = t.match(vh);
  return e ? e[1].replace(bh, "'") : t;
}
const wh = {
  name: "add",
  returnType: "number",
  schema: k({
    a: K((t) => t === null ? void 0 : t, Z.number()),
    b: K((t) => t === null ? void 0 : t, Z.number())
  })
}, kh = {
  name: "subtract",
  returnType: "number",
  schema: k({
    a: K((t) => t === null ? void 0 : t, Z.number()),
    b: K((t) => t === null ? void 0 : t, Z.number())
  })
}, $h = {
  name: "multiply",
  returnType: "number",
  schema: k({
    a: K((t) => t === null ? void 0 : t, Z.number()),
    b: K((t) => t === null ? void 0 : t, Z.number())
  })
}, Sh = {
  name: "divide",
  returnType: "number",
  schema: k({
    a: K((t) => t === null ? void 0 : t, Z.number()),
    b: K((t) => t === null ? void 0 : t, Z.number())
  })
}, xh = {
  name: "equals",
  returnType: "boolean",
  schema: k({
    a: be().refine((t) => t !== void 0, "Required"),
    b: be().refine((t) => t !== void 0, "Required")
  })
}, Ch = {
  name: "not_equals",
  returnType: "boolean",
  schema: k({
    a: be().refine((t) => t !== void 0, "Required"),
    b: be().refine((t) => t !== void 0, "Required")
  })
}, Th = {
  name: "greater_than",
  returnType: "boolean",
  schema: k({
    a: K((t) => t === null ? void 0 : t, Z.number()),
    b: K((t) => t === null ? void 0 : t, Z.number())
  })
}, Eh = {
  name: "less_than",
  returnType: "boolean",
  schema: k({
    a: K((t) => t === null ? void 0 : t, Z.number()),
    b: K((t) => t === null ? void 0 : t, Z.number())
  })
}, Oh = {
  name: "and",
  returnType: "boolean",
  schema: k({
    values: tt(be()).min(2)
  })
}, Ph = {
  name: "or",
  returnType: "boolean",
  schema: k({
    values: tt(be()).min(2)
  })
}, Dh = {
  name: "not",
  returnType: "boolean",
  schema: k({
    value: be().refine((t) => t !== void 0, "Required")
  })
}, Nh = {
  name: "contains",
  returnType: "boolean",
  schema: k({
    string: K((t) => t === void 0 ? void 0 : String(t), O()),
    substring: K((t) => t === void 0 ? void 0 : String(t), O())
  })
}, jh = {
  name: "starts_with",
  returnType: "boolean",
  schema: k({
    string: K((t) => t === void 0 ? void 0 : String(t), O()),
    prefix: K((t) => t === void 0 ? void 0 : String(t), O())
  })
}, Rh = {
  name: "ends_with",
  returnType: "boolean",
  schema: k({
    string: K((t) => t === void 0 ? void 0 : String(t), O()),
    suffix: K((t) => t === void 0 ? void 0 : String(t), O())
  })
}, Lh = {
  name: "required",
  returnType: "boolean",
  schema: k({
    value: be().refine((t) => t !== void 0, "Required")
  })
}, Mh = {
  name: "regex",
  returnType: "boolean",
  schema: k({
    value: K((t) => t === void 0 ? void 0 : String(t), O()),
    pattern: K((t) => t === void 0 ? void 0 : String(t), O())
  })
}, Fh = {
  name: "length",
  returnType: "boolean",
  schema: k({
    value: be().refine((t) => t !== void 0, "Required"),
    min: Z.number().optional(),
    max: Z.number().optional()
  }).refine((t) => t.min !== void 0 || t.max !== void 0, {
    message: "Must provide either 'min' or 'max'"
  })
}, Ih = {
  name: "numeric",
  returnType: "boolean",
  schema: k({
    value: Z.number(),
    min: Z.number().optional(),
    max: Z.number().optional()
  }).refine((t) => t.min !== void 0 || t.max !== void 0, {
    message: "Must provide either 'min' or 'max'"
  })
}, zh = {
  name: "email",
  returnType: "boolean",
  schema: k({
    value: K((t) => t === void 0 ? void 0 : String(t), O())
  })
}, Zh = {
  name: "formatString",
  returnType: "any",
  schema: k({
    value: Z.string()
  })
}, Uh = {
  name: "formatNumber",
  returnType: "string",
  schema: k({
    value: Z.number(),
    decimals: Z.number().optional(),
    grouping: et().default(!0)
  })
}, Vh = {
  name: "formatCurrency",
  returnType: "string",
  schema: k({
    value: Z.number(),
    currency: Z.string(),
    decimals: Z.number().optional(),
    grouping: et().default(!0)
  })
}, Wh = {
  name: "formatDate",
  returnType: "string",
  schema: k({
    value: be().refine((t) => t !== void 0, "Required"),
    format: Z.string()
  })
}, Bh = {
  name: "pluralize",
  returnType: "string",
  schema: k({
    value: Z.number(),
    zero: Z.string().optional(),
    one: Z.string().optional(),
    two: Z.string().optional(),
    few: Z.string().optional(),
    many: Z.string().optional(),
    other: Z.string()
  }).passthrough()
}, qh = {
  name: "openUrl",
  returnType: "void",
  schema: k({
    url: K((t) => t === void 0 ? void 0 : String(t), O())
  })
}, Hh = B(wh, (t) => t.a + t.b), Yh = B(kh, (t) => t.a - t.b), Gh = B($h, (t) => t.a * t.b), Jh = B(Sh, (t) => {
  const e = t.a, r = t.b;
  if (e == null || r === void 0 || r === null)
    return NaN;
  const n = Number(e), s = Number(r);
  return Number.isNaN(n) || Number.isNaN(s) ? NaN : s === 0 ? 1 / 0 : n / s;
}), Xh = B(xh, (t) => t.a === t.b), Qh = B(Ch, (t) => t.a !== t.b), Kh = B(Th, (t) => t.a > t.b), ep = B(Eh, (t) => t.a < t.b), tp = B(Oh, (t) => t.values.every((e) => !!e)), rp = B(Ph, (t) => t.values.some((e) => !!e)), np = B(Dh, (t) => !t.value), sp = B(Nh, (t) => t.string.includes(t.substring)), ip = B(jh, (t) => t.string.startsWith(t.prefix)), ap = B(Rh, (t) => t.string.endsWith(t.suffix)), op = B(Lh, (t) => {
  const e = t.value;
  return !(e == null || typeof e == "string" && e === "" || Array.isArray(e) && e.length === 0);
}), lp = B(Mh, (t) => {
  try {
    return new RegExp(t.pattern).test(t.value);
  } catch (e) {
    throw new Me(`Invalid regex pattern: ${t.pattern}`, "regex", e);
  }
}), cp = B(Fh, (t) => {
  const e = t.value;
  let r = 0;
  return (typeof e == "string" || Array.isArray(e)) && (r = e.length), !(t.min !== void 0 && !isNaN(t.min) && r < t.min || t.max !== void 0 && !isNaN(t.max) && r > t.max);
}), up = B(Ih, (t) => !(isNaN(t.value) || t.min !== void 0 && !isNaN(t.min) && t.value < t.min || t.max !== void 0 && !isNaN(t.max) && t.value > t.max)), dp = B(zh, (t) => /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/.test(t.value)), fp = B(Zh, (t, e) => {
  const r = t.value, s = new Ms().parse(r);
  if (s.length === 0)
    return "";
  const i = s.map((a) => typeof a != "object" || a === null || Array.isArray(a) ? a : e.resolveSignal(a));
  return Ya(() => i.map((a) => fs(a) ? a.value : a).join(""));
}), hp = B(Uh, (t) => isNaN(t.value) ? "" : new Intl.NumberFormat("en-US", {
  minimumFractionDigits: t.decimals,
  maximumFractionDigits: t.decimals,
  useGrouping: t.grouping
}).format(t.value)), pp = B(Vh, (t) => {
  if (isNaN(t.value))
    return "";
  try {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: t.currency,
      minimumFractionDigits: t.decimals,
      maximumFractionDigits: t.decimals,
      useGrouping: t.grouping
    }).format(t.value);
  } catch {
    return t.value.toFixed(t.decimals || 2);
  }
}), mp = B(Wh, (t) => {
  if (!t.value)
    return "";
  const e = new Date(t.value);
  if (isNaN(e.getTime()))
    return "";
  try {
    return t.format === "ISO" ? e.toISOString() : _h(e, t.format);
  } catch (r) {
    return console.warn("Error formatting date:", r), e.toISOString();
  }
}), gp = B(Bh, (t) => {
  const e = new Intl.PluralRules("en-US").select(t.value);
  return String(t[e] ?? t.other ?? "");
}), vp = B(qh, (t) => {
  t.url && typeof window < "u" && window.open && window.open(t.url, "_blank");
}), bp = [
  Hh,
  Yh,
  Gh,
  Jh,
  Xh,
  Qh,
  Kh,
  ep,
  tp,
  rp,
  np,
  sp,
  ip,
  ap,
  op,
  lp,
  cp,
  up,
  dp,
  fp,
  hp,
  pp,
  mp,
  gp,
  vp
], ae = {
  accessibility: Md.optional(),
  weight: sr().describe("The relative weight of this component within a Row or Column. This is similar to the CSS 'flex-grow' property. Note: this may ONLY be set when the component is a direct descendant of a Row or Column.").optional()
}, Cn = {
  name: "Text",
  schema: k({
    ...ae,
    text: H.describe("The text content to display. While simple Markdown formatting is supported (i.e. without HTML, images, or links), utilizing dedicated UI components is generally preferred for a richer and more structured presentation."),
    variant: le(["h1", "h2", "h3", "h4", "h5", "caption", "body"]).default("body").describe("A hint for the base text style.").optional()
  }).strict()
}, wo = {
  name: "Image",
  schema: k({
    ...ae,
    url: H.describe("The URL of the image to display."),
    description: H.describe("The accessibility description of the image.").optional(),
    fit: le(["contain", "cover", "fill", "none", "scaleDown"]).default("fill").describe("Specifies how the image should be resized to fit its container. This corresponds to the CSS 'object-fit' property.").optional(),
    variant: le([
      "icon",
      "avatar",
      "smallFeature",
      "mediumFeature",
      "largeFeature",
      "header"
    ]).default("mediumFeature").describe("A hint for the image size and style.").optional()
  }).strict()
}, yp = [
  "accountCircle",
  "add",
  "arrowBack",
  "arrowForward",
  "attachFile",
  "calendarToday",
  "call",
  "camera",
  "check",
  "close",
  "delete",
  "download",
  "edit",
  "event",
  "error",
  "fastForward",
  "favorite",
  "favoriteOff",
  "folder",
  "help",
  "home",
  "info",
  "locationOn",
  "lock",
  "lockOpen",
  "mail",
  "menu",
  "moreVert",
  "moreHoriz",
  "notificationsOff",
  "notifications",
  "pause",
  "payment",
  "person",
  "phone",
  "photo",
  "play",
  "print",
  "refresh",
  "rewind",
  "search",
  "send",
  "settings",
  "share",
  "shoppingCart",
  "skipNext",
  "skipPrevious",
  "star",
  "starHalf",
  "starOff",
  "stop",
  "upload",
  "visibility",
  "visibilityOff",
  "volumeDown",
  "volumeMute",
  "volumeOff",
  "volumeUp",
  "warning"
], ko = {
  name: "Icon",
  schema: k({
    ...ae,
    name: Se([
      le(yp),
      k({
        path: O()
      }).strict()
    ]).describe("The name of the icon to display.")
  }).strict()
}, $o = {
  name: "Video",
  schema: k({
    ...ae,
    url: H.describe("The URL of the video to display.")
  }).strict()
}, So = {
  name: "AudioPlayer",
  schema: k({
    ...ae,
    url: H.describe("The URL of the audio to be played."),
    description: H.describe("A description of the audio, such as a title or summary.").optional()
  }).strict()
}, Tn = {
  name: "Row",
  schema: k({
    ...ae,
    children: Os.describe("Defines the children. Use an array of strings for a fixed set of children, or a template object to generate children from a data list. Children cannot be defined inline, they must be referred to by ID."),
    justify: le([
      "center",
      "end",
      "spaceAround",
      "spaceBetween",
      "spaceEvenly",
      "start",
      "stretch"
    ]).default("start").describe("Defines the arrangement of children along the main axis (horizontally). Use 'spaceBetween' to push items to the edges, or 'start'/'end'/'center' to pack them together.").optional(),
    align: le(["start", "center", "end", "stretch"]).default("stretch").describe("Defines the alignment of children along the cross axis (vertically). This is similar to the CSS 'align-items' property, but uses camelCase values (e.g., 'start').").optional()
  }).strict().describe("A layout component that arranges its children horizontally. To create a grid layout, nest Columns within this Row.")
}, En = {
  name: "Column",
  schema: k({
    ...ae,
    children: Os.describe("Defines the children. Use an array of strings for a fixed set of children, or a template object to generate children from a data list. Children cannot be defined inline, they must be referred to by ID."),
    justify: le([
      "start",
      "center",
      "end",
      "spaceBetween",
      "spaceAround",
      "spaceEvenly",
      "stretch"
    ]).default("start").describe("Defines the arrangement of children along the main axis (vertically). Use 'spaceBetween' to push items to the edges (e.g. header at top, footer at bottom), or 'start'/'end'/'center' to pack them together.").optional(),
    align: le(["center", "end", "start", "stretch"]).default("stretch").describe("Defines the alignment of children along the cross axis (horizontally). This is similar to the CSS 'align-items' property.").optional()
  }).strict().describe("A layout component that arranges its children vertically. To create a grid layout, nest Rows within this Column.")
}, xo = {
  name: "List",
  schema: k({
    ...ae,
    children: Os.describe("Defines the children. Use an array of strings for a fixed set of children, or a template object to generate children from a data list."),
    direction: le(["vertical", "horizontal"]).default("vertical").describe("The direction in which the list items are laid out.").optional(),
    align: le(["start", "center", "end", "stretch"]).default("stretch").describe("Defines the alignment of children along the cross axis.").optional()
  }).strict()
}, Co = {
  name: "Card",
  schema: k({
    ...ae,
    child: ft.describe("The ID of the single child component to be rendered inside the card. To display multiple elements, you MUST wrap them in a layout component (like Column or Row) and pass that container's ID here. Do NOT pass multiple IDs or a non-existent ID. Do NOT define the child component inline.")
  }).strict()
}, To = {
  name: "Tabs",
  schema: k({
    ...ae,
    tabs: tt(k({
      title: H.describe("The tab title."),
      child: ft.describe("The ID of the child component. Do NOT define the component inline.")
    }).strict()).min(1).describe("An array of objects, where each object defines a tab with a title and a child component.")
  }).strict()
}, Eo = {
  name: "Modal",
  schema: k({
    ...ae,
    trigger: ft.describe("The ID of the component that opens the modal when interacted with (e.g., a button). Do NOT define the component inline."),
    content: ft.describe("The ID of the component to be displayed inside the modal. Do NOT define the component inline.")
  }).strict()
}, Oo = {
  name: "Divider",
  schema: k({
    ...ae,
    axis: le(["horizontal", "vertical"]).default("horizontal").describe("The orientation of the divider.").optional()
  }).strict()
}, On = {
  name: "Button",
  schema: k({
    ...ae,
    child: ft.describe("The ID of the child component. Use a 'Text' component for a labeled button. Only use an 'Icon' if the requirements explicitly ask for an icon-only button. Do NOT define the child component inline."),
    variant: le(["default", "primary", "borderless"]).default("default").describe("A hint for the button style. If omitted, a default button style is used. 'primary' indicates this is the main call-to-action button. 'borderless' means the button has no visual border or background, making its child content appear like a clickable link.").optional(),
    action: ao,
    checks: ur.shape.checks
  }).strict()
}, Pn = {
  name: "TextField",
  schema: k({
    ...ae,
    label: H.describe("The text label for the input field."),
    value: H.describe("The value of the text field.").optional(),
    variant: le(["longText", "number", "shortText", "obscured"]).default("shortText").describe("The type of input field to display.").optional(),
    validationRegexp: O().describe("A regular expression used for client-side validation of the input.").optional(),
    checks: ur.shape.checks
  }).strict()
}, Po = {
  name: "CheckBox",
  schema: k({
    ...ae,
    label: H.describe("The text to display next to the checkbox."),
    value: so.describe("The current state of the checkbox (true for checked, false for unchecked)."),
    checks: ur.shape.checks
  }).strict()
}, Do = {
  name: "ChoicePicker",
  schema: k({
    ...ae,
    label: H.describe("The label for the group of options.").optional(),
    variant: le(["multipleSelection", "mutuallyExclusive"]).default("mutuallyExclusive").describe("A hint for how the choice picker should be displayed and behave.").optional(),
    options: tt(k({
      label: H.describe("The text to display for this option."),
      value: O().describe("The stable value associated with this option.")
    }).strict()).describe("The list of available options to choose from."),
    value: jd.describe("The list of currently selected values. This should be bound to a string array in the data model."),
    displayStyle: le(["checkbox", "chips"]).default("checkbox").describe("The display style of the component.").optional(),
    filterable: et().default(!1).describe("If true, displays a search input to filter the options.").optional(),
    checks: ur.shape.checks
  }).strict().describe("A component that allows selecting one or more options from a list.")
}, No = {
  name: "Slider",
  schema: k({
    ...ae,
    label: H.describe("The label for the slider.").optional(),
    min: sr().default(0).describe("The minimum value of the slider.").optional(),
    max: sr().describe("The maximum value of the slider."),
    value: io.describe("The current value of the slider."),
    checks: ur.shape.checks
  }).strict()
}, jo = {
  name: "DateTimeInput",
  schema: k({
    ...ae,
    value: H.describe("The selected date and/or time value in ISO 8601 format. If not yet set, initialize with an empty string."),
    enableDate: et().default(!1).describe("If true, allows the user to select a date.").optional(),
    enableTime: et().default(!1).describe("If true, allows the user to select a time.").optional(),
    min: Se([
      H,
      O().date(),
      O().time(),
      O().datetime()
    ]).describe("The minimum allowed date/time in ISO 8601 format.").optional(),
    max: Se([
      H,
      O().date(),
      O().time(),
      O().datetime()
    ]).describe("The maximum allowed date/time in ISO 8601 format.").optional(),
    label: H.describe("The text label for the input field.").optional(),
    checks: ur.shape.checks
  }).strict()
};
var _p = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Ap = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-text")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      _p(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), Ap(n, r);
    }
    createController() {
      return new Y(this, Cn);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      switch (i.variant ?? "body") {
        case "h1":
          return $`<h1>${i.text}</h1>`;
        case "h2":
          return $`<h2>${i.text}</h2>`;
        case "h3":
          return $`<h3>${i.text}</h3>`;
        case "h4":
          return $`<h4>${i.text}</h4>`;
        case "h5":
          return $`<h5>${i.text}</h5>`;
        case "caption":
          return $`<span class="caption">${i.text}</span>`;
        default:
          return $`<p>${i.text}</p>`;
      }
    }
  }, n;
})();
const wp = {
  ...Cn,
  tagName: "a2ui-text"
};
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const Ro = { ATTRIBUTE: 1 }, Lo = (t) => (...e) => ({ _$litDirective$: t, values: e });
let Mo = class {
  constructor(e) {
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AT(e, r, n) {
    this._$Ct = e, this._$AM = r, this._$Ci = n;
  }
  _$AS(e, r) {
    return this.update(e, r);
  }
  update(e, r) {
    return this.render(...r);
  }
};
/**
 * @license
 * Copyright 2018 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const ar = Lo(class extends Mo {
  constructor(t) {
    if (super(t), t.type !== Ro.ATTRIBUTE || t.name !== "class" || t.strings?.length > 2) throw Error("`classMap()` can only be used in the `class` attribute and must be the only part in the attribute.");
  }
  render(t) {
    return " " + Object.keys(t).filter((e) => t[e]).join(" ") + " ";
  }
  update(t, [e]) {
    if (this.st === void 0) {
      this.st = /* @__PURE__ */ new Set(), t.strings !== void 0 && (this.nt = new Set(t.strings.join(" ").split(/\s/).filter((n) => n !== "")));
      for (const n in e) e[n] && !this.nt?.has(n) && this.st.add(n);
      return this.render(e);
    }
    const r = t.element.classList;
    for (const n of this.st) n in e || (r.remove(n), this.st.delete(n));
    for (const n in e) {
      const s = !!e[n];
      s === this.st.has(n) || this.nt?.has(n) || (s ? (r.add(n), this.st.add(n)) : (r.remove(n), this.st.delete(n)));
    }
    return ht;
  }
});
var kp = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, $p = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-button")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      kp(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), $p(n, r);
    }
    createController() {
      return new Y(this, On);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = i.isValid === !1, l = () => {
        !a && i.action && i.action();
      }, c = {
        "a2ui-button": !0,
        "a2ui-button-primary": i.variant === "primary",
        "a2ui-button-borderless": i.variant === "borderless"
      };
      return $`
      <button
        class=${ar(c)}
        @click=${l}
        ?disabled=${a}
      >
        ${i.child ? $`${this.renderNode(i.child)}` : C}
      </button>
    `;
    }
  }, n;
})();
const Sp = {
  ...On,
  tagName: "a2ui-button"
};
var xp = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Cp = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-textfield")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      xp(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), Cp(n, r);
    }
    createController() {
      return new Y(this, Pn);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = i.isValid === !1, l = (d) => {
        const u = d.target;
        i.setValue && i.setValue(u.value);
      }, c = {
        "a2ui-textfield": !0,
        "a2ui-textfield-invalid": a
      };
      let f = "text";
      return i.variant === "number" && (f = "number"), i.variant === "obscured" && (f = "password"), $`
      <div class="a2ui-textfield-container">
        ${i.label ? $`<label>${i.label}</label>` : C}
        ${i.variant === "longText" ? $` <textarea
              class=${ar(c)}
              .value=${i.value || ""}
              @input=${l}
              pattern=${i.validationRegexp || void 0}
            ></textarea>` : $` <input
              type=${f}
              class=${ar(c)}
              .value=${i.value || ""}
              @input=${l}
              pattern=${i.validationRegexp || void 0}
            />`}
        ${a && i.validationErrors && i.validationErrors.length > 0 ? $`<div class="a2ui-error-message">
              ${i.validationErrors[0]}
            </div>` : C}
      </div>
    `;
    }
  }, n;
})();
const Tp = {
  ...Pn,
  tagName: "a2ui-textfield"
};
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
function* zr(t, e) {
  if (t !== void 0) {
    let r = 0;
    for (const n of t) yield e(n, r++);
  }
}
/**
 * @license
 * Copyright 2018 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const Fo = "important", Ep = " !" + Fo, dr = Lo(class extends Mo {
  constructor(t) {
    if (super(t), t.type !== Ro.ATTRIBUTE || t.name !== "style" || t.strings?.length > 2) throw Error("The `styleMap` directive must be used in the `style` attribute and must be the only part in the attribute.");
  }
  render(t) {
    return Object.keys(t).reduce((e, r) => {
      const n = t[r];
      return n == null ? e : e + `${r = r.includes("-") ? r : r.replace(/(?:^(webkit|moz|ms|o)|)(?=[A-Z])/g, "-$&").toLowerCase()}:${n};`;
    }, "");
  }
  update(t, [e]) {
    const { style: r } = t.element;
    if (this.ft === void 0) return this.ft = new Set(Object.keys(e)), this.render(e);
    for (const n of this.ft) e[n] == null && (this.ft.delete(n), n.includes("-") ? r.removeProperty(n) : r[n] = null);
    for (const n in e) {
      const s = e[n];
      if (s != null) {
        this.ft.add(n);
        const i = typeof s == "string" && s.endsWith(Ep);
        n.includes("-") || i ? r.setProperty(n, i ? s.slice(0, -11) : s, i ? Fo : "") : r[n] = s;
      }
    }
    return ht;
  }
});
var Op = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Pp = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
function Dp(t) {
  switch (t) {
    case "start":
      return "flex-start";
    case "center":
      return "center";
    case "end":
      return "flex-end";
    case "spaceBetween":
      return "space-between";
    case "spaceAround":
      return "space-around";
    case "spaceEvenly":
      return "space-evenly";
    case "stretch":
      return "stretch";
    default:
      return "flex-start";
  }
}
function Np(t) {
  switch (t) {
    case "start":
      return "flex-start";
    case "center":
      return "center";
    case "end":
      return "flex-end";
    case "stretch":
      return "stretch";
    default:
      return "stretch";
  }
}
(() => {
  let t = [G("a2ui-row")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      Op(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), Pp(n, r);
    }
    createController() {
      return new Y(this, Tn);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = Array.isArray(i.children) ? i.children : [], l = {
        display: "flex",
        flexDirection: "row",
        justifyContent: Dp(i.justify),
        alignItems: Np(i.align),
        flex: i.weight !== void 0 ? String(i.weight) : "initial"
      };
      return $`
      <div class="a2ui-row" style=${dr(l)}>
        ${zr(a, (c) => $`${this.renderNode(c)}`)}
      </div>
    `;
    }
  }, n;
})();
const jp = {
  ...Tn,
  tagName: "a2ui-row"
};
var Rp = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Lp = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
function Mp(t) {
  switch (t) {
    case "start":
      return "flex-start";
    case "center":
      return "center";
    case "end":
      return "flex-end";
    case "spaceBetween":
      return "space-between";
    case "spaceAround":
      return "space-around";
    case "spaceEvenly":
      return "space-evenly";
    case "stretch":
      return "stretch";
    default:
      return "flex-start";
  }
}
function Fp(t) {
  switch (t) {
    case "start":
      return "flex-start";
    case "center":
      return "center";
    case "end":
      return "flex-end";
    case "stretch":
      return "stretch";
    default:
      return "stretch";
  }
}
(() => {
  let t = [G("a2ui-column")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      Rp(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), Lp(n, r);
    }
    createController() {
      return new Y(this, En);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = Array.isArray(i.children) ? i.children : [], l = {
        display: "flex",
        flexDirection: "column",
        justifyContent: Mp(i.justify),
        alignItems: Fp(i.align),
        flex: i.weight !== void 0 ? String(i.weight) : "initial"
      };
      return $`
      <div
        class="a2ui-column"
        style=${dr(l)}
      >
        ${zr(a, (c) => $`${this.renderNode(c)}`)}
      </div>
    `;
    }
  }, n;
})();
const Ip = {
  ...En,
  tagName: "a2ui-column"
}, zp = {
  name: "capitalize",
  returnType: "string",
  schema: k({
    value: K((t) => t === void 0 ? void 0 : String(t), O()).optional()
  })
}, Zp = B(zp, (t) => t.value ? t.value.charAt(0).toUpperCase() + t.value.slice(1) : "");
new Cs("https://a2ui.org/specification/v0_9/catalogs/minimal/minimal_catalog.json", [wp, Sp, Tp, jp, Ip], [Zp]);
var Up = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Vp = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-basic-text")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      Up(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), Vp(n, r);
    }
    createController() {
      return new Y(this, Cn);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      switch (i.variant ?? "body") {
        case "h1":
          return $`<h1>${i.text}</h1>`;
        case "h2":
          return $`<h2>${i.text}</h2>`;
        case "h3":
          return $`<h3>${i.text}</h3>`;
        case "h4":
          return $`<h4>${i.text}</h4>`;
        case "h5":
          return $`<h5>${i.text}</h5>`;
        case "caption":
          return $`<span class="a2ui-caption">${i.text}</span>`;
        default:
          return $`<p>${i.text}</p>`;
      }
    }
  }, n;
})();
const Wp = {
  ...Cn,
  tagName: "a2ui-basic-text"
};
var Bp = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, qp = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-basic-button")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      Bp(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), qp(n, r);
    }
    createController() {
      return new Y(this, On);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = i.isValid === !1, l = {
        "a2ui-button": !0,
        ["a2ui-button-" + (i.variant || "default")]: !0
      };
      return $`
      <button
        class=${ar(l)}
        @click=${() => !a && i.action && i.action()}
        ?disabled=${a}
      >
        ${i.child ? $`${this.renderNode(i.child)}` : C}
      </button>
    `;
    }
  }, n;
})();
const Hp = {
  ...On,
  tagName: "a2ui-basic-button"
};
var Yp = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Gp = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-basic-textfield")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      Yp(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), Gp(n, r);
    }
    createController() {
      return new Y(this, Pn);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = i.isValid === !1, l = (d) => i.setValue?.(d.target.value);
      let c = "text";
      i.variant === "number" && (c = "number"), i.variant === "obscured" && (c = "password");
      const f = { "a2ui-textfield": !0, invalid: a };
      return $`
      <div class="a2ui-textfield-container">
        ${i.label ? $`<label>${i.label}</label>` : C}
        ${i.variant === "longText" ? $`<textarea
              class=${ar(f)}
              .value=${i.value || ""}
              @input=${l}
            ></textarea>` : $`<input
              type=${c}
              class=${ar(f)}
              .value=${i.value || ""}
              @input=${l}
            />`}
        ${a && i.validationErrors?.length ? $`<div class="error">${i.validationErrors[0]}</div>` : C}
      </div>
    `;
    }
  }, n;
})();
const Jp = {
  ...Pn,
  tagName: "a2ui-basic-textfield"
};
var Xp = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Qp = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-basic-row")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      Xp(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), Qp(n, r);
    }
    createController() {
      return new Y(this, Tn);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = Array.isArray(i.children) ? i.children : [], l = {
        display: "flex",
        flexDirection: "row",
        flex: i.weight !== void 0 ? String(i.weight) : "initial",
        gap: "8px"
      };
      return $`
      <div class="a2ui-row" style=${dr(l)}>
        ${zr(a, (c) => $`${this.renderNode(c)}`)}
      </div>
    `;
    }
  }, n;
})();
const Kp = {
  ...Tn,
  tagName: "a2ui-basic-row"
};
var em = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, tm = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-basic-column")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      em(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), tm(n, r);
    }
    createController() {
      return new Y(this, En);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = Array.isArray(i.children) ? i.children : [], l = {
        display: "flex",
        flexDirection: "column",
        flex: i.weight !== void 0 ? String(i.weight) : "initial",
        gap: "8px"
      };
      return $`
      <div
        class="a2ui-column"
        style=${dr(l)}
      >
        ${zr(a, (c) => $`${this.renderNode(c)}`)}
      </div>
    `;
    }
  }, n;
})();
const rm = {
  ...En,
  tagName: "a2ui-basic-column"
};
var nm = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, sm = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-list")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      nm(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), sm(n, r);
    }
    createController() {
      return new Y(this, xo);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = Array.isArray(i.children) ? i.children : [], l = {
        display: "flex",
        flexDirection: i.direction === "horizontal" ? "row" : "column",
        overflow: "auto",
        gap: "8px"
      };
      return $`
      <div
        class="a2ui-list"
        style=${dr(l)}
      >
        ${zr(a, (c) => $`${this.renderNode(c)}`)}
      </div>
    `;
    }
  }, n;
})();
const im = {
  ...xo,
  tagName: "a2ui-list"
};
var am = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, om = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-image")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      am(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), om(n, r);
    }
    createController() {
      return new Y(this, wo);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = { objectFit: i.fit || "fill", width: "100%" };
      return $`<img
      src=${i.url}
      alt=${i.description || ""}
      class=${"a2ui-image " + (i.variant || "")}
      style=${dr(a)}
    />`;
    }
  }, n;
})();
const lm = {
  ...wo,
  tagName: "a2ui-image"
};
var cm = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, um = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-icon")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      cm(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), um(n, r);
    }
    createController() {
      return new Y(this, ko);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = typeof i.name == "string" ? i.name : i.name?.path;
      return $`<span class="material-symbols-outlined a2ui-icon"
      >${a}</span
    >`;
    }
  }, n;
})();
const dm = {
  ...ko,
  tagName: "a2ui-icon"
};
var fm = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, hm = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-video")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      fm(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), hm(n, r);
    }
    createController() {
      return new Y(this, $o);
    }
    render() {
      const i = this.controller.props;
      return i ? $`<video src=${i.url} controls class="a2ui-video"></video>` : C;
    }
  }, n;
})();
const pm = {
  ...$o,
  tagName: "a2ui-video"
};
var mm = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, gm = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-audioplayer")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      mm(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), gm(n, r);
    }
    createController() {
      return new Y(this, So);
    }
    render() {
      const i = this.controller.props;
      return i ? $`<div class="a2ui-audioplayer">
      ${i.description ? $`<p>${i.description}</p>` : C}
      <audio src=${i.url} controls></audio>
    </div>` : C;
    }
  }, n;
})();
const vm = {
  ...So,
  tagName: "a2ui-audioplayer"
};
var bm = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, ym = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-card")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      bm(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), ym(n, r);
    }
    createController() {
      return new Y(this, Co);
    }
    render() {
      const i = this.controller.props;
      return i ? $`
      <div
        class="a2ui-card"
        style="border: 1px solid #ccc; border-radius: 8px; padding: 16px;"
      >
        ${i.child ? $`${this.renderNode(i.child)}` : C}
      </div>
    ` : C;
    }
  }, n;
})();
const _m = {
  ...Co,
  tagName: "a2ui-card"
};
var Am = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, wm = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-divider")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      Am(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), wm(n, r);
    }
    createController() {
      return new Y(this, Oo);
    }
    render() {
      const i = this.controller.props;
      return i ? i.axis === "vertical" ? $`<div
          class="a2ui-divider-vertical"
          style="width: 1px; background: #ccc; height: 100%;"
        ></div>` : $`<hr
          class="a2ui-divider"
          style="border: none; border-top: 1px solid #ccc; margin: 16px 0;"
        />` : C;
    }
  }, n;
})();
const km = {
  ...Oo,
  tagName: "a2ui-divider"
};
var $m = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Sm = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-checkbox")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      $m(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), Sm(n, r);
    }
    createController() {
      return new Y(this, Po);
    }
    render() {
      const i = this.controller.props;
      return i ? $`
      <label class="a2ui-checkbox">
        <input
          type="checkbox"
          .checked=${i.value || !1}
          @change=${(a) => i.setValue?.(a.target.checked)}
        />
        ${i.label}
      </label>
    ` : C;
    }
  }, n;
})();
const xm = {
  ...Po,
  tagName: "a2ui-checkbox"
};
var Cm = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Tm = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-slider")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      Cm(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), Tm(n, r);
    }
    createController() {
      return new Y(this, No);
    }
    render() {
      const i = this.controller.props;
      return i ? $`
      <div class="a2ui-slider">
        ${i.label ? $`<label>${i.label}</label>` : C}
        <input
          type="range"
          min=${i.min ?? 0}
          max=${i.max ?? 100}
          .value=${i.value?.toString() || "0"}
          @input=${(a) => i.setValue?.(Number(a.target.value))}
        />
        <span>${i.value}</span>
      </div>
    ` : C;
    }
  }, n;
})();
const Em = {
  ...No,
  tagName: "a2ui-slider"
};
var Om = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Pm = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-datetimeinput")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      Om(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), Pm(n, r);
    }
    createController() {
      return new Y(this, jo);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = i.enableDate && i.enableTime ? "datetime-local" : i.enableDate ? "date" : "time";
      return $`
      <div class="a2ui-datetime">
        ${i.label ? $`<label>${i.label}</label>` : C}
        <input
          type=${a}
          .value=${i.value || ""}
          @input=${(l) => i.setValue?.(l.target.value)}
        />
      </div>
    `;
    }
  }, n;
})();
const Dm = {
  ...jo,
  tagName: "a2ui-datetimeinput"
};
var Nm = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, jm = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-choicepicker")], e, r = [], n, s = J;
  return class extends s {
    static {
      n = this;
    }
    static {
      const i = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      Nm(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: i }, null, r), n = e.value, i && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: i }), jm(n, r);
    }
    createController() {
      return new Y(this, Do);
    }
    render() {
      const i = this.controller.props;
      if (!i)
        return C;
      const a = Array.isArray(i.value) ? i.value : [], l = i.variant === "multipleSelection", c = (f) => {
        i.setValue && (l ? a.includes(f) ? i.setValue(a.filter((d) => d !== f)) : i.setValue([...a, f]) : i.setValue([f]));
      };
      return $`
      <div class="a2ui-choicepicker">
        ${i.label ? $`<label>${i.label}</label>` : C}
        <div class="options">
          ${i.options?.map((f) => $`
              <label>
                <input
                  type=${l ? "checkbox" : "radio"}
                  .checked=${a.includes(f.value)}
                  @change=${() => c(f.value)}
                />
                ${f.label}
              </label>
            `)}
        </div>
      </div>
    `;
    }
  }, n;
})();
const Rm = {
  ...Do,
  tagName: "a2ui-choicepicker"
};
var Wi = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Hn = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-tabs")], e, r = [], n, s = J, i, a = [], l = [];
  return class extends s {
    static {
      n = this;
    }
    static {
      const c = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      i = [ho()], Wi(this, null, i, { kind: "accessor", name: "activeIndex", static: !1, private: !1, access: { has: (f) => "activeIndex" in f, get: (f) => f.activeIndex, set: (f, d) => {
        f.activeIndex = d;
      } }, metadata: c }, a, l), Wi(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: c }, null, r), n = e.value, c && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: c }), Hn(n, r);
    }
    createController() {
      return new Y(this, To);
    }
    #e = Hn(this, a, 0);
    get activeIndex() {
      return this.#e;
    }
    set activeIndex(c) {
      this.#e = c;
    }
    render() {
      const c = this.controller.props;
      return !c || !c.tabs ? C : $`
      <div class="a2ui-tabs">
        <div
          class="a2ui-tab-headers"
          style="display:flex; gap: 8px; border-bottom: 1px solid #ccc; margin-bottom: 16px;"
        >
          ${c.tabs.map((f, d) => $`
              <button
                @click=${() => this.activeIndex = d}
                style="padding: 8px; background: ${d === this.activeIndex ? "#eee" : "transparent"}; border: none;"
              >
                ${f.title}
              </button>
            `)}
        </div>
        <div class="a2ui-tab-content">
          ${c.tabs[this.activeIndex] ? $`${this.renderNode(c.tabs[this.activeIndex].child)}` : C}
        </div>
      </div>
    `;
    }
    constructor() {
      super(...arguments), Hn(this, l);
    }
  }, n;
})();
const Lm = {
  ...To,
  tagName: "a2ui-tabs"
};
var Bi = function(t, e, r, n, s, i) {
  function a(y) {
    if (y !== void 0 && typeof y != "function") throw new TypeError("Function expected");
    return y;
  }
  for (var l = n.kind, c = l === "getter" ? "get" : l === "setter" ? "set" : "value", f = !e && t ? n.static ? t : t.prototype : null, d = e || (f ? Object.getOwnPropertyDescriptor(f, n.name) : {}), u, o = !1, b = r.length - 1; b >= 0; b--) {
    var v = {};
    for (var g in n) v[g] = g === "access" ? {} : n[g];
    for (var g in n.access) v.access[g] = n.access[g];
    v.addInitializer = function(y) {
      if (o) throw new TypeError("Cannot add initializers after decoration has completed");
      i.push(a(y || null));
    };
    var m = (0, r[b])(l === "accessor" ? { get: d.get, set: d.set } : d[c], v);
    if (l === "accessor") {
      if (m === void 0) continue;
      if (m === null || typeof m != "object") throw new TypeError("Object expected");
      (u = a(m.get)) && (d.get = u), (u = a(m.set)) && (d.set = u), (u = a(m.init)) && s.unshift(u);
    } else (u = a(m)) && (l === "field" ? s.unshift(u) : d[c] = u);
  }
  f && Object.defineProperty(f, n.name, d), o = !0;
}, Yn = function(t, e, r) {
  for (var n = arguments.length > 2, s = 0; s < e.length; s++)
    r = n ? e[s].call(t, r) : e[s].call(t);
  return n ? r : void 0;
};
(() => {
  let t = [G("a2ui-modal")], e, r = [], n, s = J, i, a = [], l = [];
  return class extends s {
    static {
      n = this;
    }
    static {
      const c = typeof Symbol == "function" && Symbol.metadata ? Object.create(s[Symbol.metadata] ?? null) : void 0;
      i = [df("dialog")], Bi(this, null, i, { kind: "accessor", name: "dialog", static: !1, private: !1, access: { has: (f) => "dialog" in f, get: (f) => f.dialog, set: (f, d) => {
        f.dialog = d;
      } }, metadata: c }, a, l), Bi(null, e = { value: n }, t, { kind: "class", name: n.name, metadata: c }, null, r), n = e.value, c && Object.defineProperty(n, Symbol.metadata, { enumerable: !0, configurable: !0, writable: !0, value: c }), Yn(n, r);
    }
    createController() {
      return new Y(this, Eo);
    }
    #e = Yn(this, a, void 0);
    get dialog() {
      return this.#e;
    }
    set dialog(c) {
      this.#e = c;
    }
    render() {
      const c = this.controller.props;
      return c ? $`
      <div @click=${() => this.dialog?.showModal()}>
        ${c.trigger ? $`${this.renderNode(c.trigger)}` : C}
      </div>
      <dialog
        class="a2ui-modal"
        style="border: 1px solid #ccc; border-radius: 8px; padding: 24px; min-width: 300px;"
      >
        <form method="dialog" style="text-align: right;">
          <button>×</button>
        </form>
        ${c.content ? $`${this.renderNode(c.content)}` : C}
      </dialog>
    ` : C;
    }
    constructor() {
      super(...arguments), Yn(this, l);
    }
  }, n;
})();
const Mm = {
  ...Eo,
  tagName: "a2ui-modal"
}, Io = new Cs("https://a2ui.org/specification/v0_9/basic_catalog.json", [
  Wp,
  Hp,
  Jp,
  Kp,
  rm,
  im,
  lm,
  dm,
  pm,
  vm,
  _m,
  km,
  xm,
  Em,
  Dm,
  Rm,
  Lm,
  Mm
], bp), zo = "https://focusa.dev/a2ui/v0_9/catalog.json", Fm = k({
  label: H.optional(),
  description: H.optional(),
  status: H.optional(),
  progress: io.optional(),
  primaryActionLabel: H.optional(),
  action: ao.optional(),
  disabled: et().optional(),
  busy: et().optional(),
  details: H.optional()
}).strict();
za.map(({ name: t }) => t);
const Zo = [];
for (const t of za) {
  const e = `${t.tag}-a2ui`, r = { name: t.name, schema: Fm }, n = mo(t.tag);
  customElements.get(e) || customElements.define(e, class extends J {
    createController() {
      return new Y(this, r);
    }
    render() {
      const s = this.controller.props;
      return s ? go`<${n}
          .label=${s.label ?? t.name}
          .description=${s.description ?? ""}
          .status=${s.status ?? "ready"}
          .progress=${s.progress ?? 0}
          .primaryActionLabel=${s.primaryActionLabel ?? "Continue"}
          .actionAvailable=${typeof s.action == "function"}
          .disabled=${s.disabled ?? !1}
          .busy=${s.busy ?? !1}
          .details=${s.details ?? ""}
          .invokeAction=${s.action}
        ></${n}>` : C;
    }
  }), Zo.push({ ...r, tagName: e });
}
const qi = new Cs(zo, [
  ...Io.components.values(),
  ...Zo
]), Im = "v0.9", zm = {
  maxMessages: 256,
  maxSerializedBytes: 1e6
};
class Zm {
  processor;
  limits;
  #e = /* @__PURE__ */ new Map();
  #t = /* @__PURE__ */ new Set();
  #r = new Set(qi.components.keys());
  #a;
  #n;
  constructor(e = {}) {
    this.limits = { ...zm, ...e.limits }, this.#a = e.onAction, this.#n = e.allowedActionNames ?? /* @__PURE__ */ new Set(), this.processor = new Pd(
      [Io, qi],
      (r) => this.dispatchAction(r)
    );
  }
  processSnapshot(e) {
    if (!e.some((r) => "createSurface" in r))
      throw new Error("A2UI snapshot must create at least one surface");
    this.#i(e);
  }
  processDelta(e) {
    this.#i(e);
  }
  mount(e, r) {
    const n = this.processor.model.getSurface(r);
    if (!n) throw new Error(`Unknown A2UI surface: ${r}`);
    this.#e.get(r)?.element.remove();
    const i = document.createElement("a2ui-surface");
    return i.surface = n, e.replaceChildren(i), this.#e.set(r, { container: e, element: i }), i;
  }
  async dispatchAction(e) {
    const r = e.name;
    if (this.#n.has(r) && this.#a) {
      await this.#a(e);
      return;
    }
    this.#l(
      e.surfaceId,
      `Action ${r} is unavailable or outside the generated Operation Registry binding.`
    );
  }
  surfaceIds() {
    return [...this.processor.model.surfacesMap.keys()].sort();
  }
  clientCapabilities() {
    return this.processor.getClientCapabilities({ includeInlineCatalogs: !1 });
  }
  dispose() {
    for (const { container: e } of this.#e.values()) e.replaceChildren();
    this.#e.clear(), this.#t.clear(), this.processor.model.dispose();
  }
  #i(e) {
    if (e.length === 0 || e.length > this.limits.maxMessages)
      throw new Error(`A2UI message count must be 1-${this.limits.maxMessages}`);
    if (new TextEncoder().encode(JSON.stringify(e)).byteLength > this.limits.maxSerializedBytes)
      throw new Error(`A2UI payload exceeds ${this.limits.maxSerializedBytes} bytes`);
    const n = [];
    for (const s of e) {
      if (s.version !== Im)
        throw new Error(`Unsupported A2UI protocol version: ${String(s.version)}`);
      "createSurface" in s && s.createSurface.catalogId === zo && this.#t.add(s.createSurface.surfaceId), "deleteSurface" in s && this.#t.delete(s.deleteSurface.surfaceId), n.push(this.#s(s));
    }
    this.processor.processMessages(n);
  }
  #s(e) {
    if (!("updateComponents" in e)) return e;
    const r = e.updateComponents;
    if (!this.#t.has(r.surfaceId)) return e;
    const n = r.components.map((s) => {
      const i = s, a = String(i.component ?? "");
      return this.#r.has(a) ? s : {
        id: String(i.id ?? "unsupported"),
        component: "FocusaRecoveryCard",
        label: "Unsupported generated component",
        description: `${a || "Unknown component"} is not in the trusted Focusa catalog.`,
        status: "recovery",
        details: "No action was executed. Refresh the surface or use the recovery action."
      };
    });
    return { ...e, updateComponents: { ...r, components: n } };
  }
  #l(e, r) {
    const n = this.#e.get(e);
    if (!n) return;
    const s = document.createElement("focusa-recovery-card");
    s.label = "Unsupported action", s.description = r, s.status = "recovery", s.details = "No action was executed. Refresh permissions or regenerate the surface.", n.container.append(s);
  }
}
class Um extends HTMLElement {
  #e;
  #t = /* @__PURE__ */ new Set();
  #r = "";
  #a = !1;
  set allowedActions(e) {
    this.#t = new Set(e), this.#e = void 0, this.#a = !1;
  }
  set snapshot(e) {
    this.#n(), this.#e.processSnapshot(e), this.#i(e);
  }
  set delta(e) {
    this.#n(), this.#e.processDelta(e), this.#a || this.#i(e);
  }
  connectedCallback() {
    this.setAttribute("role", "region"), this.hasAttribute("aria-label") || this.setAttribute("aria-label", "Generated Focusa surface");
  }
  #n() {
    this.#e || (this.#e = new Zm({
      allowedActionNames: this.#t,
      onAction: (e) => {
        this.dispatchEvent(
          new CustomEvent("focusa-operation", {
            bubbles: !0,
            composed: !0,
            detail: e
          })
        );
      }
    }));
  }
  #i(e) {
    const r = e.find((n) => "createSurface" in n);
    this.#r = r?.createSurface?.surfaceId || this.#r, this.#r && (this.#e.mount(this, this.#r), this.#a = !0);
  }
}
customElements.get("focusa-generated-surface") || customElements.define("focusa-generated-surface", Um);
export {
  Um as FocusaGeneratedSurfaceElement
};
