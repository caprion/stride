package app.stride.running.setup

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory
import org.w3c.dom.Element

/**
 * The copy, as it is shipped.
 *
 * These are the words the athlete reads, so they are asserted here rather
 * than described: the file is read, the strings are pulled out of it, and
 * the sentences are compared with the ones the contract names.
 */
class SetupResourcesTest {

    /** The strings file, read from the module. */
    private val strings: Map<String, String> by lazy { readStrings() }

    /** The plurals: the name, the quantity, the text. */
    private val plurals: Map<String, Map<String, String>> by lazy { readPlurals() }

    @Test
    fun `the states say what the contract says`() {
        assertEquals("Checking…", strings["garmin_checking"])
        assertEquals("Syncing…", strings["garmin_syncing"])
        assertEquals("Garmin did not accept this key.", strings["garmin_refused"])
        assertEquals("Garmin is not connected.", strings["garmin_not_connected"])
        assertEquals("Connected", strings["garmin_connected"])
        assertEquals("Garmin could not be reached.", strings["garmin_failed"])
    }

    @Test
    fun `the privacy line is one line`() {
        assertEquals("Your data stays on this phone.", strings["garmin_privacy"])
    }

    @Test
    fun `the count is a count`() {
        assertEquals("%1\$d run", plurals["garmin_runs"]?.get("one"))
        assertEquals("%1\$d runs", plurals["garmin_runs"]?.get("other"))
        assertEquals("Connected · %1\$d run", plurals["garmin_connected_runs"]?.get("one"))
        assertEquals("Connected · %1\$d runs", plurals["garmin_connected_runs"]?.get("other"))
    }

    @Test
    fun `the screen says 12 runs and then 160 runs`() {
        // The count as it arrives, and the count when it is over.
        assertEquals("12 runs", count("garmin_runs", 12))
        assertEquals("160 runs", count("garmin_runs", 160))
        assertEquals("Connected · 160 runs", count("garmin_connected_runs", 160))
        assertEquals("Connected · 1 run", count("garmin_connected_runs", 1))
    }

    @Test
    fun `the copy on the Garmin screen is short`() {
        // Every word on the screen, and the longest of them.
        val onScreen = listOf(
            "garmin_title",
            "garmin_key_hint",
            "garmin_show_key",
            "garmin_open_garmin",
            "garmin_connect",
            "garmin_replace_key",
            "garmin_disconnect",
            "garmin_not_now",
            "garmin_not_connected",
            "garmin_connected",
            "garmin_checking",
            "garmin_syncing",
            "garmin_refused",
            "garmin_failed",
            "garmin_key_too_short",
            "garmin_no_browser",
            "garmin_credential_refused",
            "garmin_privacy",
        )
        for (name in onScreen) {
            val text = strings[name] ?: throw AssertionError("the string $name is gone")
            assertTrue(
                "\"$name\" is ${text.length} characters long — the screen's copy is short",
                text.length <= 60,
            )
        }
    }

    @Test
    fun `the server field and the long copy are gone`() {
        // The phone has no server, so there is no URL on the screen and no
        // string for one.
        for (name in listOf(
            "server_url_label",
            "server_url_hint",
            "server_url_save",
            "server_url_saved",
            "server_url_cleared",
            "server_url_must_be_https",
        )) {
            assertFalse("the string $name is still there", strings.containsKey(name))
        }

        // The paragraph that explained what local-first means.
        assertFalse(strings.containsKey("garmin_explanation"))

        // The toasts are gone: the screen says what happened.
        assertFalse(strings.containsKey("garmin_connected_toast"))
        assertFalse(strings.containsKey("garmin_disconnected_toast"))
    }

    @Test
    fun `the layout has no server field`() {
        val layout = readLayout()
        assertFalse("the layout still has a server field", layout.contains("server_url"))
    }

    /** The count, as the screen renders it. */
    private fun count(name: String, quantity: Int): String {
        val items = plurals[name] ?: throw AssertionError("the plural $name is gone")
        val text = items["one"] ?: throw AssertionError("the plural $name has no singular")
        val template = if (quantity == 1) text else items["other"] ?: text
        return template.replace("%1\$d", quantity.toString())
    }

    /** The string resources, by name. */
    private fun readStrings(): Map<String, String> {
        val document = document()
        val nodes = document.getElementsByTagName("string")
        val found = LinkedHashMap<String, String>()
        for (index in 0 until nodes.length) {
            val element = nodes.item(index) as Element
            found[element.getAttribute("name")] = element.textContent
        }
        return found
    }

    /** The plural resources, by name and by quantity. */
    private fun readPlurals(): Map<String, Map<String, String>> {
        val document = document()
        val nodes = document.getElementsByTagName("plurals")
        val found = LinkedHashMap<String, Map<String, String>>()
        for (index in 0 until nodes.length) {
            val element = nodes.item(index) as Element
            val items = LinkedHashMap<String, String>()
            val children = element.getElementsByTagName("item")
            for (child in 0 until children.length) {
                val item = children.item(child) as Element
                items[item.getAttribute("quantity")] = item.textContent
            }
            found[element.getAttribute("name")] = items
        }
        return found
    }

    /** The layout of the setup screen. */
    private fun readLayout(): String =
        file("src/main/res/layout/activity_garmin_setup.xml").readText()

    /** The strings file. */
    private fun document() = DocumentBuilderFactory.newInstance()
        .newDocumentBuilder()
        .parse(file("src/main/res/values/strings.xml"))

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
