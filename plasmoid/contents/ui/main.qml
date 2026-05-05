// meowtrics KDE Plasma 6 plasmoid root.
//
// Plasma 6 imports must be unversioned. Versioned imports
// ("org.kde.plasma.plasmoid 2.0") silently load a Plasma 5-shaped
// PlasmoidItem without toolTipMainText / toolTipSubText.
//
// State source: `meowtrics tray-state` is a pre-classified JSON dump
// from the daemon CLI. The plasmoid is a dumb renderer.

import QtQuick
import QtQuick.Layouts
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.components as PC3
import org.kde.plasma.plasma5support as P5Support

PlasmoidItem {
    id: root

    // Pre-classified state from `meowtrics tray-state`. Names mirror
    // src/sprites.rs::animation_for(): sleep / sit_calm / sit_alert /
    // wash_face / run_panic / scratch.
    property string activeAnimation: "sit_calm"
    property string headline: "starting up"
    property var sensors: []

    Plasmoid.icon: "meowtrics"
    toolTipMainText: "meowtrics"
    toolTipSubText: root.headline

    compactRepresentation: CompactRepresentation {
        animation: root.activeAnimation
    }

    fullRepresentation: FullRepresentation {
        sensors: root.sensors
        headline: root.headline
        animation: root.activeAnimation
    }

    P5Support.DataSource {
        id: trayStateRunner
        engine: "executable"
        connectedSources: []
        onNewData: function (sourceName, data) {
            disconnectSource(sourceName);
            const stdout = data["stdout"];
            if (!stdout) return;
            try {
                const obj = JSON.parse(stdout);
                if (obj.animation) root.activeAnimation = obj.animation;
                if (obj.headline)  root.headline       = obj.headline;
                if (obj.sensors)   root.sensors        = obj.sensors;
            } catch (e) {
                root.headline = "could not parse meowtrics tray-state";
            }
        }
        function fetch() { connectSource("meowtrics tray-state"); }
    }

    Timer {
        // Pull a fresh classification every 5 s; the compact rep handles
        // its own intra-state frame animation at a much faster timer.
        interval: 5000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: trayStateRunner.fetch()
    }
}
