package app.stride.running.bridge

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The route table's rules, tested without a phone.
 *
 * The plan's acceptance check for the Connect screen is one line: *"The key
 * never appears in any WebView-reachable surface: asserted by a test over
 * the bridge's route table."* This is that test.
 */
class PhoneRoutesTest {

    /** The frozen route table, as the plan writes it. */
    private val frozen = listOf(
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

    @Test
    fun `every route of the frozen table is known`() {
        for (route in frozen) {
            val decision = PhoneRoutes.decide("GET", route)
            assertTrue("$route is not known", decision.known)
        }
    }

    @Test
    fun `a route that is not in the frozen table is refused`() {
        val refused = listOf(
            "/network/settings",
            "/tiles/proxy",
            "/gear",
            "/auth/garmin/login",
            "/auth/garmin/mfa",
            "/auth/garmin/developer-key",
            "/auth/garmin/callback",
            "/auth/pair-code",
            "/auth/pair",
            "/auth/devices",
            "/stats/monthly",
            "/stats/hr-zones",
            "/",
            "",
        )
        for (route in refused) {
            assertFalse("$route is known", PhoneRoutes.decide("GET", route).known)
        }
    }

    @Test
    fun `an id is one path segment and nothing more`() {
        assertTrue(PhoneRoutes.decide("GET", "/activities/17").known)
        assertFalse(PhoneRoutes.decide("GET", "/activities/17/laps").known)
        assertFalse(PhoneRoutes.decide("GET", "/activities/17/17").known)
    }

    /**
     * The key routes are refused whatever the method is, and whatever is
     * appended to the path.
     *
     * **The query string is the whole point.** A guard that is decided on the
     * raw path is a guard that `/auth/garmin/personal-key?x=1` turns off:
     * the router splits the path at the `?` and matches what is left, so
     * the request would reach the key route while the guard thought it was
     * looking at something else.
     */
    @Test
    fun `the key routes are refused, including the query-string forms`() {
        val keyRoutes = listOf(
            "/auth/garmin/personal-key",
            "/auth/garmin/personal-key/test",
        )
        for (route in keyRoutes) {
            for (method in listOf("GET", "POST", "PUT", "DELETE")) {
                for (path in listOf(
                    route,
                    "$route?x=1",
                    "$route?",
                    "$route?key=abc",
                    "/api/v1$route",
                    "/api/v1$route?x=1",
                )) {
                    val decision = PhoneRoutes.decide(method, path)
                    assertTrue("$method $path carries the key", decision.carriesKey)
                    assertEquals(
                        "$method $path was classified wrongly",
                        PhoneRoutes.Kind.GarminKey,
                        decision.kind,
                    )
                }
            }
        }
    }

    @Test
    fun `the query string is not part of the route`() {
        assertEquals("/activities", PhoneRoutes.routeOf("/activities?limit=10"))
        assertEquals("/activities", PhoneRoutes.routeOf("/activities?"))
        assertEquals("/activities", PhoneRoutes.routeOf("/activities#top"))
        assertEquals("/activities", PhoneRoutes.routeOf("/api/v1/activities"))
        assertEquals("/activities", PhoneRoutes.routeOf("/api/v1/activities?limit=10"))
    }

    @Test
    fun `the api prefix is only a prefix when it is a whole segment`() {
        assertEquals("/activities", PhoneRoutes.routeOf("/api/v1/activities"))
        assertEquals("/api/v10/activities", PhoneRoutes.routeOf("/api/v10/activities"))
    }

    /** The plan's acceptance check, over the whole table. */
    @Test
    fun `no route of the frozen table carries the key to the page`() {
        for (route in frozen) {
            for (method in listOf("GET", "POST", "PUT", "DELETE")) {
                val decision = PhoneRoutes.decide(method, route)
                val carriesKey = decision.carriesKey
                val isKeyRoute = route.startsWith("/auth/garmin/personal-key")
                assertEquals(
                    "$method $route was classified wrongly",
                    isKeyRoute,
                    carriesKey,
                )
            }
        }
    }

    @Test
    fun `a pull is a pull and a logout is a logout`() {
        assertEquals(
            PhoneRoutes.Kind.Pull,
            PhoneRoutes.decide("POST", "/sync/run").kind,
        )
        assertEquals(
            PhoneRoutes.Kind.Logout,
            PhoneRoutes.decide("POST", "/auth/garmin/logout").kind,
        )
        assertEquals(
            PhoneRoutes.Kind.Data,
            PhoneRoutes.decide("GET", "/activities").kind,
        )
        assertEquals(
            PhoneRoutes.Kind.Write,
            PhoneRoutes.decide("POST", "/plan/import").kind,
        )
    }

    @Test
    fun `a status route carries no athlete data`() {
        val decision = PhoneRoutes.decide("GET", "/health")
        assertEquals(PhoneRoutes.Kind.Status, decision.kind)
        assertFalse(decision.carriesData)
    }

    @Test
    fun `a read of the athlete's own history carries data`() {
        val decision = PhoneRoutes.decide("GET", "/activities")
        assertTrue(decision.carriesData)
    }
}
