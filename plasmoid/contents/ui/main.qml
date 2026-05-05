// meowtrics KDE Plasma 6 plasmoid root.
//
// Thin client over the meowtrics daemon. The daemon owns sensor reading,
// state machines, and message selection; the plasmoid renders the active
// emoji in the panel and a popup on click.
//
// State is pulled by running `meowtrics json` on a timer (Plasma 6 QML's
// Plasma5Support.DataSource ProcessRunner). The richer D-Bus path is the
// v0.2 plan once the rendering is solid.

import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.plasmoid 2.0
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.components 3.0 as PlasmaComponents
import org.kde.plasma.plasma5support 2.0 as P5Support

PlasmoidItem {
    id: root

    property string activeEmoji: "🐈"
    property string headline: "starting up"
    property var sensors: []

    Plasmoid.icon: Qt.resolvedUrl("../icons/meowtrics.png")
    toolTipMainText: "meowtrics"
    toolTipSubText: root.headline

    compactRepresentation: CompactRepresentation {
        emoji: root.activeEmoji
        headline: root.headline
    }

    fullRepresentation: FullRepresentation {
        sensors: root.sensors
        activeEmoji: root.activeEmoji
        headline: root.headline
    }

    P5Support.DataSource {
        id: jsonRunner
        engine: "executable"
        connectedSources: []
        onNewData: function (sourceName, data) {
            disconnectSource(sourceName);
            const stdout = data["stdout"];
            if (!stdout) return;
            try {
                const arr = JSON.parse(stdout);
                root.sensors = arr;
                if (arr.length > 0) {
                    // Pick the highest "value" sensor as the active one for the panel.
                    let pick = arr[0];
                    for (const s of arr) if ((s.value || 0) > (pick.value || 0)) pick = s;
                    // Tray emoji + headline updates would normally come from the daemon
                    // through D-Bus. Until that's wired, the JSON-only path keeps the
                    // sensor table in the popup live but leaves the panel emoji static.
                    root.headline = pick.sensor + " " + Math.round(pick.value);
                }
            } catch (e) {
                root.headline = "could not parse meowtrics json";
            }
        }
        function fetch() { connectSource("meowtrics json"); }
    }

    Timer {
        interval: 5000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: jsonRunner.fetch()
    }
}
