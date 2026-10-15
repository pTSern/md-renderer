package com.mdviewer.app

import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.OpenableColumns
import android.util.Log
import android.webkit.*
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.io.File
import java.io.InputStreamReader

class MainActivity : AppCompatActivity() {

    private lateinit var webView: WebView
    private var pendingSaveContent: String? = null
    private var initialFileUri: Uri? = null
    private var filePathCallback: ValueCallback<Array<Uri>>? = null

    private val openDocumentLauncher = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) {
            loadFileFromUri(uri)
        }
    }

    private val createDocumentLauncher = registerForActivityResult(ActivityResultContracts.CreateDocument("text/markdown")) { uri ->
        if (uri != null) {
            val contentToSave = pendingSaveContent ?: ""
            saveContentToUri(uri, contentToSave)
            pendingSaveContent = null
        }
    }

    private val fileChooserLauncher = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        if (filePathCallback != null) {
            val results = WebChromeClient.FileChooserParams.parseResult(result.resultCode, result.data)
            filePathCallback?.onReceiveValue(results)
            filePathCallback = null
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Capture intent file URI if opened from external app / file manager
        handleIntent(intent)

        setupWebView()

        // Handle hardware / gesture back button
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                webView.evaluateJavascript(
                    """
                    (function() {
                        if (typeof TableOfContents !== 'undefined' && TableOfContents.isOpen) {
                            TableOfContents.close();
                            return 'closed_toc';
                        }
                        if (typeof AiAgent !== 'undefined' && AiAgent.isOpen) {
                            AiAgent.close();
                            return 'closed_agent';
                        }
                        const popover = document.getElementById('translation-popover');
                        if (popover && popover.classList.contains('show')) {
                            popover.classList.remove('show');
                            return 'closed_trans';
                        }
                        const kbModal = document.getElementById('keybindings-modal');
                        if (kbModal && kbModal.classList.contains('show')) {
                            kbModal.classList.remove('show');
                            return 'closed_modal';
                        }
                        return 'none';
                    })();
                    """.trimIndent()
                ) { result ->
                    val cleanResult = result?.replace("\"", "") ?: "none"
                    if (cleanResult == "none") {
                        if (webView.canGoBack()) {
                            webView.goBack()
                        } else {
                            isEnabled = false
                            onBackPressedDispatcher.onBackPressed()
                        }
                    }
                }
            }
        })
    }

    override fun onNewIntent(intent: Intent?) {
        super.onNewIntent(intent)
        setIntent(intent)
        intent?.let { handleIntent(it) }
        initialFileUri?.let { loadFileFromUri(it) }
    }

    private fun handleIntent(intent: Intent) {
        if (intent.action == Intent.ACTION_VIEW) {
            initialFileUri = intent.data
        }
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun setupWebView() {
        webView = WebView(this)
        setContentView(webView)

        webView.settings.apply {
            javaScriptEnabled = true
            domStorageEnabled = true
            databaseEnabled = true
            allowFileAccess = true
            allowContentAccess = true
            allowFileAccessFromFileURLs = true
            allowUniversalAccessFromFileURLs = true
            useWideViewPort = true
            loadWithOverviewMode = true
            mediaPlaybackRequiresUserGesture = false
            cacheMode = WebSettings.LOAD_DEFAULT
        }

        webView.addJavascriptInterface(AndroidBridge(), "AndroidBridge")

        webView.webChromeClient = object : WebChromeClient() {
            override fun onConsoleMessage(consoleMessage: ConsoleMessage?): Boolean {
                consoleMessage?.let {
                    Log.d("MDViewerJS", "${it.message()} -- From line ${it.lineNumber()} of ${it.sourceId()}")
                }
                return true
            }

            override fun onShowFileChooser(
                webView: WebView?,
                filePathCallback: ValueCallback<Array<Uri>>?,
                fileChooserParams: FileChooserParams?
            ): Boolean {
                this@MainActivity.filePathCallback?.onReceiveValue(null)
                this@MainActivity.filePathCallback = filePathCallback

                val intent = fileChooserParams?.createIntent() ?: Intent(Intent.ACTION_GET_CONTENT).apply {
                    type = "*/*"
                    addCategory(Intent.CATEGORY_OPENABLE)
                }

                try {
                    fileChooserLauncher.launch(intent)
                } catch (e: Exception) {
                    this@MainActivity.filePathCallback = null
                    return false
                }
                return true
            }
        }

        webView.webViewClient = object : WebViewClient() {
            override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
                val url = request?.url?.toString() ?: return false
                if (url.startsWith("http://") || url.startsWith("https://")) {
                    try {
                        val browserIntent = Intent(Intent.ACTION_VIEW, Uri.parse(url))
                        startActivity(browserIntent)
                        return true
                    } catch (e: Exception) {
                        Log.e("MDViewer", "Failed to launch browser", e)
                    }
                }
                return false
            }
        }

        webView.loadUrl("file:///android_asset/app.html")
    }

    private fun loadFileFromUri(uri: Uri) {
        lifecycleScope.launch(Dispatchers.IO) {
            try {
                var fileName = "Document.md"
                contentResolver.query(uri, null, null, null, null)?.use { cursor ->
                    val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    if (nameIndex != -1 && cursor.moveToFirst()) {
                        fileName = cursor.getString(nameIndex)
                    }
                }

                val content = contentResolver.openInputStream(uri)?.use { stream ->
                    InputStreamReader(stream, Charsets.UTF_8).readText()
                } ?: ""

                val response = JSONObject().apply {
                    put("type", "file_opened")
                    put("path", uri.toString())
                    put("name", fileName)
                    put("content", content)
                }

                withContext(Dispatchers.Main) {
                    sendToWeb(response.toString())
                }
            } catch (e: Exception) {
                Log.e("MDViewer", "Error reading file from URI: $uri", e)
            }
        }
    }

    private fun saveContentToUri(uri: Uri, content: String) {
        lifecycleScope.launch(Dispatchers.IO) {
            try {
                contentResolver.openOutputStream(uri, "wt")?.use { stream ->
                    stream.write(content.toByteArray(Charsets.UTF_8))
                    stream.flush()
                }

                val response = JSONObject().apply {
                    put("type", "file_saved")
                    put("path", uri.toString())
                }

                withContext(Dispatchers.Main) {
                    sendToWeb(response.toString())
                }
            } catch (e: Exception) {
                Log.e("MDViewer", "Error saving file to URI: $uri", e)
            }
        }
    }

    private fun sendToWeb(jsonString: String) {
        val escaped = JSONObject.quote(jsonString)
        webView.evaluateJavascript("if (window.receiveFromAndroid) { window.receiveFromAndroid($escaped); }", null)
    }

    inner class AndroidBridge {

        @JavascriptInterface
        fun postMessage(jsonString: String) {
            try {
                val data = JSONObject(jsonString)
                val cmd = data.optString("cmd")

                when (cmd) {
                    "get_init_data" -> {
                        lifecycleScope.launch(Dispatchers.IO) {
                            var initialContent = ""
                            var initialPath = ""
                            var initialName = "sample.md"

                            if (initialFileUri != null) {
                                initialPath = initialFileUri.toString()
                                contentResolver.query(initialFileUri!!, null, null, null, null)?.use { cursor ->
                                    val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                                    if (nameIndex != -1 && cursor.moveToFirst()) {
                                        initialName = cursor.getString(nameIndex)
                                    }
                                }
                                initialContent = contentResolver.openInputStream(initialFileUri!!)?.use {
                                    InputStreamReader(it, Charsets.UTF_8).readText()
                                } ?: ""
                            } else {
                                try {
                                    assets.open("sample.md").use { stream ->
                                        initialContent = InputStreamReader(stream, Charsets.UTF_8).readText()
                                    }
                                } catch (_: Exception) {
                                    initialContent = "# Welcome to MDViewer\n\nStart typing your markdown here."
                                }
                            }

                            // Load keybindings
                            val prefs = getSharedPreferences("mdviewer_prefs", Context.MODE_PRIVATE)
                            var kbJson = prefs.getString("keybindings_json", null)
                            if (kbJson == null) {
                                try {
                                    assets.open("keybindings.json").use { stream ->
                                        kbJson = InputStreamReader(stream, Charsets.UTF_8).readText()
                                    }
                                } catch (_: Exception) {
                                    kbJson = "{}"
                                }
                            }

                            val response = JSONObject().apply {
                                put("type", "init")
                                put("path", initialPath)
                                put("name", initialName)
                                put("content", initialContent)
                                put("json", kbJson)
                            }

                            withContext(Dispatchers.Main) {
                                sendToWeb(response.toString())
                            }
                        }
                    }

                    "open_file" -> {
                        openDocumentLauncher.launch(arrayOf("text/*", "application/*", "*/*"))
                    }

                    "save_file" -> {
                        val path = data.optString("path", "")
                        val content = data.optString("content", "")
                        if (path.isNotEmpty() && (path.startsWith("content://") || path.startsWith("file://"))) {
                            try {
                                val uri = Uri.parse(path)
                                saveContentToUri(uri, content)
                            } catch (e: Exception) {
                                pendingSaveContent = content
                                createDocumentLauncher.launch("document.md")
                            }
                        } else {
                            pendingSaveContent = content
                            createDocumentLauncher.launch("document.md")
                        }
                    }

                    "save_file_as" -> {
                        val content = data.optString("content", "")
                        val defaultName = data.optString("defaultName", "document.md")
                        pendingSaveContent = content
                        createDocumentLauncher.launch(defaultName)
                    }

                    "save_keybindings" -> {
                        val json = data.optString("json", "")
                        getSharedPreferences("mdviewer_prefs", Context.MODE_PRIVATE)
                            .edit()
                            .putString("keybindings_json", json)
                            .apply()

                        val resp = JSONObject().apply {
                            put("type", "keybindings_saved")
                        }
                        sendToWeb(resp.toString())
                    }

                    "save_ai_chats" -> {
                        val json = data.optString("json", "[]")
                        try {
                            val chatsFile = File(filesDir, "ai_chats.json")
                            chatsFile.writeText(json, Charsets.UTF_8)
                        } catch (e: Exception) {
                            Log.e("MDViewer", "Failed to save AI chats", e)
                        }
                    }

                    "get_ai_chats" -> {
                        lifecycleScope.launch(Dispatchers.IO) {
                            var chatsJson = "[]"
                            try {
                                val chatsFile = File(filesDir, "ai_chats.json")
                                if (chatsFile.exists()) {
                                    chatsJson = chatsFile.readText(Charsets.UTF_8)
                                }
                            } catch (e: Exception) {
                                Log.e("MDViewer", "Failed to read AI chats", e)
                            }
                            val resp = JSONObject().apply {
                                put("type", "ai_chats_data")
                                put("json", chatsJson)
                            }
                            withContext(Dispatchers.Main) {
                                sendToWeb(resp.toString())
                            }
                        }
                    }

                    "test_ai_env_var" -> {
                        val varName = data.optString("var_name", "")
                        val value = System.getenv(varName)
                        val ok = !value.isNullOrBlank()
                        val resp = JSONObject().apply {
                            put("type", "ai_env_var_result")
                            put("var_name", varName)
                            put("ok", ok)
                            put("value", value ?: "")
                        }
                        sendToWeb(resp.toString())
                    }

                    "set_title" -> {
                        val titleText = data.optString("title", "MDViewer")
                        runOnUiThread {
                            title = titleText
                        }
                    }

                    "close_window" -> {
                        runOnUiThread {
                            finish()
                        }
                    }

                    else -> {
                        Log.d("MDViewer", "Unhandled AndroidBridge cmd: $cmd")
                    }
                }
            } catch (e: Exception) {
                Log.e("MDViewer", "Failed to parse message: $jsonString", e)
            }
        }
    }
}
