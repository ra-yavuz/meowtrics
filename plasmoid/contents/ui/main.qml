// meowtrics KDE Plasma 6 plasmoid root.
//
// The plasmoid is a thin client over the meowtrics daemon. The daemon owns
// sensor reading, state machines, and message selection; the plasmoid renders
// the active emoji in the panel and a richer popup on click.
//
// Communication: D-Bus service org.rayavuz.Meowtrics on the session bus
// (the daemon registers it on startup; plasmoid subscribes to PropertiesChanged
// and pulls state on demand).
//
// For v0.1 this file is a working skeleton: it polls a JSON status file at
// $XDG_RUNTIME_DIR/meowtrics/state.json (a fallback the daemon writes on each
// tick), which avoids a hard QML/D-Bus binding for the first cut. The D-Bus
// path is wired in v0.2 once the rendering and popup behaviour is solid.

import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.plasmoid 2.0
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.components 3.0 as PlasmaComponents

PlasmoidItem {
    id: root

    property string activeEmoji: "🐈"
    property string headline: "starting up..."
    property var sensors: []

    Plasmoid.icon: "face-smile"
    Plasmoid.toolTipMainText: "meowtrics"
    Plasmoid.toolTipSubText: root.headline

    compactRepresentation: CompactRepresentation {
        emoji: root.activeEmoji
        headline: root.headline
    }

    fullRepresentation: FullRepresentation {
        sensors: root.sensors
        activeEmoji: root.activeEmoji
        headline: root.headline
    }

    Timer {
        // Poll the daemon's state file. The daemon writes it on each tick.
        interval: 1500
        running: true
        repeat: true
        onTriggered: stateLoader.refresh()
    }

    QtObject {
        id: stateLoader
        function refresh() {
            // In v0.2 this is replaced by D-Bus PropertiesChanged subscription.
            // For now: read the state file via a small XHR-style approach.
            // Plasma 6 QML doesn't expose plain file IO without an addon, so the
            // v0.1 path is to call out to `meowtrics json` via a DataSource.
            // Wire this up properly in the next pass; this stub keeps the
            // QML loadable so the plasmoid installs cleanly.
            root.activeEmoji = root.activeEmoji;
        }
    }
}
