package app.stride.running.bridge

/**
 * What the phone does with a route.
 *
 * The routing itself lives in Rust — the router owns the path table and the
 * response shapes. What lives *here* is the small set of decisions that are
 * the phone's and not the server's:
 *
 * * the Garmin key is entered in the app, so the route that would carry it
 *   opens the native screen instead;
 * * the routes the phone does not have (a Garmin password, an OAuth
 *   developer key) are refused with the reason, before anything is sent;
 * * a pull needs the Garmin key, and the pull itself is the phone's
 *   background job.
 *
 * It is a plain object with no Android in it, so it can be tested without a
 * phone.
 */
object PhoneRoutes {

    /** What the phone does with a request. */
    enum class Decision {
        /** Straight to the core. */
        Forward,

        /** The route would carry the Garmin key: the native screen answers it. */
        GarminKeyEntry,

        /** The Garmin account flow: the phone has no password and stores none. */
        GarminLogin,

        /** The developer-key flow: the phone has no OAuth client. */
        GarminDeveloperKey,

        /** A pull from Garmin. */
        GarminPull,
    }

    /**
     * The route a path names.
     *
     * **The query string is not part of the route.** The router splits the
     * path at the `?` and matches what is left, so
     * `/api/v1/auth/garmin/personal-key?x=1` is the
     * `/api/v1/auth/garmin/personal-key` route. A decision made on the raw
     * path would therefore be made on a *different* route from the one the
     * router runs — and a decision that is wrong in that direction is a
     * guard that a query string turns off.
     *
     * So the path is cut here, once, and every decision is made on the
     * route.
     */
    fun routeOf(path: String): String {
        val cut = path.indexOf('?')
        return if (cut < 0) path else path.substring(0, cut)
    }

    /**
     * What the phone does with a request.
     *
     * The method matters: the same path read and written are different
     * things, and only the written ones carry a credential.
     */
    fun decide(method: String, path: String): Decision {
        val post = method.equals("POST", ignoreCase = true)
        if (!post) return Decision.Forward

        return when (routeOf(path)) {
            "/api/v1/auth/garmin/personal-key",
            "/api/v1/auth/garmin/personal-key/test" -> Decision.GarminKeyEntry

            "/api/v1/auth/garmin/login",
            "/api/v1/auth/garmin/mfa" -> Decision.GarminLogin

            "/api/v1/auth/garmin/developer-key",
            "/api/v1/auth/garmin/exchange" -> Decision.GarminDeveloperKey

            "/api/v1/sync/run" -> Decision.GarminPull

            else -> Decision.Forward
        }
    }
}
