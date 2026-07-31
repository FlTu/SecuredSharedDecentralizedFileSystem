package com.syfi.app

import android.os.Bundle
import android.widget.ArrayAdapter
import android.widget.Button
import android.widget.EditText
import android.widget.ListView
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

/**
 * Explorateur minimal (docs/015-roadmap.md Phase 9) : ouvre/cree un coffre,
 * liste le contenu de sa racine. Pas de montage systeme reel — Android ne
 * permet pas de FUSE utilisateur sans root, c'est cette approche
 * (explorateur interne) qui reste la seule realiste (cf. 006-storage.md,
 * 015-roadmap.md §5).
 */
class MainActivity : AppCompatActivity() {

    private var vaultHandle: Long = 0

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        val etPath = findViewById<EditText>(R.id.et_path)
        val etPassphrase = findViewById<EditText>(R.id.et_passphrase)
        val tvStatus = findViewById<TextView>(R.id.tv_status)
        val listEntries = findViewById<ListView>(R.id.list_entries)

        findViewById<Button>(R.id.btn_create).setOnClickListener {
            val ok = NativeBridge.nativeCreateVault(
                etPath.text.toString(),
                etPassphrase.text.toString()
            )
            tvStatus.text = if (ok) "Coffre cree." else "Echec de creation (existe deja ?)."
        }

        findViewById<Button>(R.id.btn_open).setOnClickListener {
            if (vaultHandle != 0L) {
                NativeBridge.nativeCloseVault(vaultHandle)
                vaultHandle = 0
            }
            vaultHandle = NativeBridge.nativeOpenVault(
                etPath.text.toString(),
                etPassphrase.text.toString()
            )
            if (vaultHandle == 0L) {
                tvStatus.text = "Echec d'ouverture (mauvaise passphrase ou chemin invalide)."
                listEntries.adapter = null
                return@setOnClickListener
            }
            tvStatus.text = "Coffre ouvert."

            val raw = NativeBridge.nativeListRoot(vaultHandle)
            val lines = raw.lineSequence().filter { it.isNotBlank() }.map { line ->
                val parts = line.split("\t")
                val kind = if (parts.getOrNull(0) == "dir") "📁" else "📄"
                val name = parts.getOrNull(1) ?: "?"
                val size = parts.getOrNull(2) ?: "0"
                "$kind $name ($size octets)"
            }.toList()

            listEntries.adapter = ArrayAdapter(
                this,
                android.R.layout.simple_list_item_1,
                lines
            )
        }
    }

    override fun onDestroy() {
        if (vaultHandle != 0L) {
            NativeBridge.nativeCloseVault(vaultHandle)
            vaultHandle = 0
        }
        super.onDestroy()
    }
}
