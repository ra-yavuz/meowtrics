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

    // Custom tooltip: a big animated cat + headline + sensor table. Set
    // toolTipItem on PlasmoidItem to override the default text-only tooltip.
    toolTipItem: ToolTipContent {
        animation: root.activeAnimation
        headline: root.headline
        sensors: root.sensors
    }

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
            const stderr = data["stderr"];
            const exit   = data["exit code"];
            if (exit !== undefined && exit !== 0) {
                console.warn("meowtrics: tray-state exited", exit, "stderr=", stderr);
                root.headline = "meowtrics daemon failed (exit " + exit + ")";
                return;
            }
            if (!stdout) {
                console.warn("meowtrics: tray-state stdout empty; stderr=", stderr);
                return;
            }
            // Robustness: take everything from the FIRST `{` to the LAST `}`.
            // The previous version used lastIndexOf("{") which captured only
            // the last sensor sub-object, breaking the JSON.
            const start = stdout.indexOf("{");
            const end   = stdout.lastIndexOf("}");
            if (start < 0 || end < 0 || end < start) {
                console.warn("meowtrics: no JSON found in stdout=", stdout);
                root.headline = "meowtrics: empty tray-state output";
                return;
            }
            const json = stdout.substring(start, end + 1);
            try {
                const obj = JSON.parse(json);
                if (obj.animation) root.activeAnimation = obj.animation;
                if (obj.headline)  root.headline       = obj.headline;
                if (obj.sensors)   root.sensors        = obj.sensors;
            } catch (e) {
                console.warn("meowtrics: JSON.parse failed:", e, "input=", json);
                root.headline = "meowtrics: JSON parse failed";
            }
        }
        // Redirect stderr to /dev/null at the shell level so any log noise
        // the daemon emits (or that wraps things like sudo/locale warnings)
        // never lands in the same data["stdout"] field that some
        // Plasma5Support executable engine builds merge.
        function fetch() { connectSource("meowtrics tray-state 2>/dev/null"); }
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
