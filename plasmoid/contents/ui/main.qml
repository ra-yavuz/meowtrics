// meowtrics KDE Plasma 6 plasmoid root.
//
// Plasma 6 imports do NOT use version suffixes; that was the Plasma 5
// pattern. Using "org.kde.plasma.plasmoid 2.0" silently downgrades the
// loaded module so PlasmoidItem.toolTipMainText / toolTipSubText do not
// exist.

import QtQuick
import QtQuick.Layouts
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.components as PC3
import org.kde.plasma.plasma5support as P5Support

PlasmoidItem {
    id: root

    property string activeEmoji: "🐈"
    property string headline: "starting up"
    property var sensors: []

    Plasmoid.icon: "meowtrics"
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
                    let pick = arr[0];
                    for (const s of arr) if ((s.value || 0) > (pick.value || 0)) pick = s;
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
