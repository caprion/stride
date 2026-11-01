package app.stride.running.bridge

/**
 * What the phone does with a route.
 *
 * The routing itself lives in Rust — the router owns the path table and the
 * response shapes. What lives *here* is the small set of decisions that are
 * the phone's and not the server's:
 *
 * * the bridge may only call the **frozen route table** — anything else is
 *   refused before anything is sent;
 * * the Garmin key is entered in the app, so the routes that would carry it
 *   are answered by the native screen and never reach the page;
 * * a pull needs the Garmin key, and the pull itself is the phone's
 *   background job;
 * * a locked screen gets no real data.
 *
 * It is a plain object with no Android in it, so it can be tested without a
 * phone.
 */
object PhoneRoutes {

    /**
     * The prefix the desktop's router may put in front of a path.
     *
     * The frozen route table (`docs/contract/api-paths.md`) names the routes
     * without it, and the phone may be asked for either form. The prefix is
     * cut here, once, so that every decision below is made on the same
     * route the router matches on.
     */
    const val API_PREFIX = "/api/v1"

    /**
     * The route that brings a file in: `POST /auth/garmin/fit-import`.
     */
    const val FIT_IMPORT = "/auth/garmin/fit-import"

    /**
     * The route that takes the Garmin key in.
     */
    const val PERSONAL_KEY = "/auth/garmin/personal-key"

    /**
     * The route that checks the Garmin key.
     */
    const val PERSONAL_KEY_TEST = "/auth/garmin/personal-key/test"

    /**
     * The route that starts a pull from Garmin.
     */
    const val SYNC_RUN = "/sync/run"

    /**
     * The route that forgets the Garmin key.
     */
    const val GARMIN_LOGOUT = "/auth/garmin/logout"

    /**
     * What a route is, for the phone's purposes.
     */
    enum class Kind {
        /** A liveness or status line. It carries no athlete's data. */
        Status,

        /** A read of the athlete's own history. */
        Data,

        /** A write of the athlete's own history. */
        Write,

        /** A route that would carry the Garmin key. */
        GarminKey,

        /** A route that brings a file in. */
        FitImport,

        /** A pull from Garmin. */
        Pull,

        /** The route that forgets the Garmin key. */
        Logout,
    }

    /**
     * A route, and what the phone does with it.
     */
    data class Decision(
        /** The route, without the query string and without the prefix. */
        val route: String,
        /** What the route is. */
        val kind: Kind,
        /** Whether the route is in the frozen table at all. */
        val known: Boolean,
    ) {
        /** Whether this route carries or writes the athlete's own data. */
        val carriesData: Boolean
            get() = kind == Kind.Data || kind == Kind.Write ||
                kind == Kind.FitImport || kind == Kind.Pull

        /** Whether this route would carry the Garmin key. */
        val carriesKey: Boolean
            get() = kind == Kind.GarminKey
    }

    /**
     * The route a path names.
     *
     * **The query string is not part of the route.** The router splits the
     * path at the `?` and matches what is left, so
     * `/auth/garmin/personal-key?x=1` is the
     * `/auth/garmin/personal-key` route. A decision made on the raw path
     * would therefore be made on a *different* route from the one the
     * router runs — and a decision that is wrong in that direction is a
     * guard that a query string turns off.
     *
     * So the path is cut here, once, and every decision is made on the
     * route.
     */
    fun routeOf(path: String): String {
        val cut = path.indexOfAny(charArrayOf('?', '#'))
        val withoutQuery = if (cut < 0) path else path.substring(0, cut)
        val trimmed = withoutQuery.trim()
        if (trimmed.length > API_PREFIX.length && trimmed.startsWith(API_PREFIX)) {
            val rest = trimmed.substring(API_PREFIX.length)
            // Only a whole path segment is a prefix: "/api/v10/activities"
            // is not "/activities".
            if (rest.startsWith("/")) return rest
        }
        return trimmed
    }

    /**
     * The frozen route table.
     *
     * Every route of `docs/contract/api-paths.md`, and nothing else. The
     * bridge may call these and no others: a path that is not in this table
     * is refused before it reaches the core, so the WebView cannot use the
     * bridge to reach anything the router does not have.
     *
     * The table is frozen, so it is copied here and not derived: a route
     * that appeared or vanished because a core crate changed would be a
     * change to a contract, and contracts are not changed by the shell.
     *
     * Two of the routes are worth naming:
     *
     * * `/fitness/fingerprint` is in the table and has no screen: the data
     *   is already synced, and the route is a documented endpoint with no
     *   UI.
     * * `/physiology/vitals` is not `/physiology/readiness`: v2 has no
     *   readiness score, so the route answers with the vitals and their
     *   directions.
     */
    private val TABLE: List<String> = listOf(
        // Data
        "/activities",
        "/activities/{id}",
        "/physiology/daily",
        "/physiology/sleep",
        "/physiology/hrv",
        "/physiology/vitals",
        "/physiology/latest",
        "/physiology/sources",
        "/fitness/fingerprint",
        "/fitness/trends",
        "/fitness/zones",
        "/stats/summary",
        "/stats/weekly",
        "/stats/pace-trends",
        "/stats/elevation",
        "/stats/personal-records",
        // Plan and insight
        "/plan/current",
        "/plan/today",
        "/plan/week",
        "/plan/blocks",
        "/plan/sources",
        "/plan/adherence",
        "/plan/results",
        "/plan/import",
        "/insights/today",
        "/insights/progress",
        "/insights/chips",
        "/compare",
        "/compare/candidates",
        // App
        "/health",
        "/health/daily",
        "/sync/status",
        "/sync/run",
        "/sync/runs",
        "/sync/history",
        "/sync/settings",
        "/auth/status",
        "/auth/garmin/personal-key",
        "/auth/garmin/personal-key/test",
        "/auth/garmin/fit-import",
        "/auth/garmin/logout",
        "/calendar",
    )

    /**
     * Whether a route is in the frozen table.
     *
     * `{id}` stands for one path segment, so `/activities/17` is the
     * `/activities/{id}` route and `/activities/17/laps` is not a route at
     * all.
     */
    fun isKnown(route: String): Boolean = TABLE.any { pattern ->
        if (!pattern.contains('{')) {
            pattern == route
        } else {
            val head = pattern.substringBefore('{').trimEnd('/')
            val tail = pattern.substringAfter('}').substringAfter('/')
            route.startsWith(head + "/") &&
                !route.removePrefix(head + "/").contains('/') &&
                (tail.isEmpty() || route.endsWith("/$tail"))
        }
    }

    /**
     * What the phone does with a request.
     *
     * The method matters: the same path read and written are different
     * things, and only the written ones carry a credential.
     */
    fun decide(method: String, path: String): Decision {
        val route = routeOf(path)
        val post = method.equals("POST", ignoreCase = true)

        if (!isKnown(route)) {
            return Decision(route, Kind.Status, known = false)
        }

        val kind = when {
            // The key routes are refused whatever the method is. A read of
            // the key route would answer with the key, and a write of it
            // would carry the key into the page: neither may happen.
            route == PERSONAL_KEY || route == PERSONAL_KEY_TEST -> Kind.GarminKey
            route == FIT_IMPORT -> Kind.FitImport
            route == GARMIN_LOGOUT -> Kind.Logout
            route == SYNC_RUN && post -> Kind.Pull
            isStatus(route) -> Kind.Status
            // Anything that is not a read is a write. The frozen table does
            // not name the methods, so the safe reading is the one that
            // asks for the lock: a request that changes something is
            // treated as a write.
            !method.equals("GET", ignoreCase = true) -> Kind.Write
            else -> Kind.Data
        }
        return Decision(route, kind, known = true)
    }

    /**
     * The routes that answer with a status rather than with the athlete's
     * own data.
     */
    private fun isStatus(route: String): Boolean = route in setOf(
        "/health",
        "/health/daily",
        "/auth/status",
        "/sync/status",
        "/sync/settings",
    )
}
