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
     * What the phone does with a request.
     *
     * The method matters: the same path read and written are different
     * things, and only the written ones carry a credential.
     */
    fun decide(method: String, path: String): Decision {
        val post = method.equals("POST", ignoreCase = true)
        if (!post) return Decision.Forward

        return when (path) {
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
