package app.stride.running.setup

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory
import org.w3c.dom.Element

/**
 * The Garmin screen's own shape, asserted rather than described.
 *
 * The theme, the one link and the URL it goes to are the three things that
 * are easy to break and hard to see: a screen that inherits the launch
 * theme gets an overlay over it, and a link that opens a plain browser loses
 * the field the athlete was filling in.
 */
class GarminSetupScreenTest {

    @Test
    fun `the link carries the one instruction that matters`() {
        assertEquals(
            "Get a key from Garmin — pick the one-year expiry",
            strings["garmin_open_garmin"],
        )
    }

    @Test
    fun `the button says save and not replace`() {
        // "Replace" reads as destructive. The athlete is saving a key.
        assertEquals("Save key", strings["garmin_save_key"])
        assertFalse(
            "the string still says replace",
            strings.containsKey("garmin_replace_key"),
        )
    }

    @Test
    fun `the link is directly under the key field`() {
        val layout = layout()
        val field = elementWithId(layout, "garmin_key")
        val next = nextElement(field)
        assertEquals(
            "the thing under the key field is not the Garmin link",
            "garmin_open_garmin",
            idOf(next),
        )
        assertEquals(
            "the link is a link and not a button",
            "TextView",
            next.tagName,
        )
    }

    @Test
    fun `there is one link and it is not in a row with another button`() {
        val layout = layout()
        assertEquals(
            "there is more than one Garmin link on the screen",
            1,
            countOfLinks(layout),
        )
        // No horizontal row on the screen at all: the link is on its own
        // line, under the field.
        val rows = layout.getElementsByTagName("LinearLayout")
        for (index in 0 until rows.length) {
            val row = rows.item(index) as Element
            assertEquals(
                "there is a row of buttons on the screen",
                "vertical",
                row.getAttribute("android:orientation").ifEmpty { "vertical" },
            )
        }
    }

    @Test
    fun `the activity has its own theme`() {
        val manifest = document("src/main/AndroidManifest.xml")
        val activity = activityElement(manifest)
        assertEquals(
            "the activity still takes the application's theme",
            "@style/AppTheme.GarminSetup",
            activity.getAttribute("android:theme"),
        )
    }

    @Test
    fun `the theme is plain and has no splash in it`() {
        val styles = document("src/main/res/values/styles.xml")
        val style = styleElement(styles, "AppTheme.GarminSetup")
        val parent = style.getAttribute("parent")
        assertTrue(
            "the theme is not an AppCompat theme: $parent",
            parent.startsWith("Theme.AppCompat"),
        )
        assertFalse(
            "the theme is a splash theme: $parent",
            parent.contains("SplashScreen"),
        )
        // Nothing in it brings a splash back.
        for (index in 0 until style.childNodes.length) {
            val child = style.childNodes.item(index)
            if (child is Element) {
                val name = child.getAttribute("name")
                assertFalse(
                    "the theme has a splash attribute in it: $name",
                    name.contains("SplashScreen") || name.contains("splashScreen"),
                )
            }
        }
        // And it is the app's surface colour.
        assertTrue(
            "the theme does not carry the app's surface colour",
            style.textContent.contains("@color/stride_surface"),
        )
    }

    @Test
    fun `the Garmin page is Garmin's own key page`() {
        assertEquals(
            "https://connect.garmin.com/app/settings/personalApiToken",
            GarminKeyPage.URL,
        )
    }

    @Test
    fun `the link opens in a Custom Tab`() {
        val source = file("src/main/java/app/stride/running/setup/GarminSetupActivity.kt")
            .readText()
        assertTrue(
            "the link does not open in a Custom Tab",
            source.contains("CustomTabsIntent"),
        )
        assertTrue(
            "the Custom Tab is not launched",
            source.contains("launchUrl"),
        )
    }

    @Test
    fun `the screen is still secure`() {
        val source = file("src/main/java/app/stride/running/setup/GarminSetupActivity.kt")
            .readText()
        assertTrue(
            "the credential screen lost FLAG_SECURE",
            source.contains("FLAG_SECURE"),
        )
    }

    /** How many links to Garmin the screen has. */
    private fun countOfLinks(layout: Element): Int {
        val nodes = layout.getElementsByTagName("TextView")
        var count = 0
        for (index in 0 until nodes.length) {
            val element = nodes.item(index) as Element
            if (idOf(element) == "garmin_open_garmin") count++
        }
        return count
    }

    /** An element of the layout, by its id. */
    private fun elementWithId(layout: Element, id: String): Element {
        val nodes = layout.getElementsByTagName("*")
        for (index in 0 until nodes.length) {
            val element = nodes.item(index) as Element
            if (idOf(element) == id) return element
        }
        throw AssertionError("there is no element called $id in the layout")
    }

    /** The id of an element, or an empty string. */
    private fun idOf(element: Element): String {
        val raw = element.getAttribute("android:id")
        return if (raw.startsWith("@+id/")) raw.removePrefix("@+id/") else raw
    }

    /** The element after another one. */
    private fun nextElement(element: Element): Element {
        var node = element.nextSibling
        while (node != null && node.nodeType != org.w3c.dom.Node.ELEMENT_NODE) {
            node = node.nextSibling
        }
        return node as Element
    }

    /** The Garmin activity, as the manifest declares it. */
    private fun activityElement(manifest: Element): Element {
        val activities = manifest.getElementsByTagName("activity")
        for (index in 0 until activities.length) {
            val activity = activities.item(index) as Element
            if (activity.getAttribute("android:name").contains("GarminSetupActivity")) {
                return activity
            }
        }
        throw AssertionError("the manifest does not declare the Garmin setup activity")
    }

    /** A style, by name. */
    private fun styleElement(styles: Element, name: String): Element {
        val nodes = styles.getElementsByTagName("style")
        for (index in 0 until nodes.length) {
            val style = nodes.item(index) as Element
            if (style.getAttribute("name") == name) return style
        }
        throw AssertionError("there is no style called $name")
    }

    /** The layout of the setup screen. */
    private fun layout(): Element = document("src/main/res/layout/activity_garmin_setup.xml")

    /** The string resources, by name. */
    private val strings: Map<String, String> by lazy {
        val document = document("src/main/res/values/strings.xml")
        val nodes = document.getElementsByTagName("string")
        val found = LinkedHashMap<String, String>()
        for (index in 0 until nodes.length) {
            val element = nodes.item(index) as Element
            found[element.getAttribute("name")] = element.textContent
        }
        found
    }

    /** A file of the module, parsed. */
    private fun document(path: String): Element = DocumentBuilderFactory.newInstance()
        .newDocumentBuilder()
        .parse(file(path))
        .documentElement

    /** A file of the module, found from wherever the test is run. */
    private fun file(path: String): File {
        var directory: File? = File("").absoluteFile
        while (directory != null) {
            val candidate = File(directory, path)
            if (candidate.isFile) return candidate
            directory = directory.parentFile
        }
        throw AssertionError("the file $path is not in the module")
    }
}
