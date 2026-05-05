// What's shown in the panel slot: the active Oneko sprite at panel size,
// tinted to the panel's foreground colour.
//
// Animation table mirrors SpriteLibrary::load() in src/sprites.rs so the
// panel widget and the SNI tray icon stay in sync.

import QtQuick
import Qt5Compat.GraphicalEffects
import org.kde.plasma.core as PlasmaCore
import org.kde.kirigami as Kirigami
import org.kde.plasma.plasmoid

Item {
    id: compact

    // Animation name set by main.qml from the daemon's JSON output.
    // sleep / sit_calm / sit_alert / wash_face / run_panic / scratch.
    property string animation: "sit_calm"

    readonly property string spriteDir: "file:///usr/share/meowtrics/icons/neko/"

    // Mirrors SpriteLibrary::load() in src/sprites.rs.
    readonly property var animations: ({
        "sleep":     { frames: ["sleep1", "sleep2"],                   frameMs: 800 },
        "sit_calm":  { frames: ["mati2", "mati2", "mati2", "mati3"],   frameMs: 600 },
        "sit_alert": { frames: ["awake"],                              frameMs: 1500 },
        "wash_face": { frames: ["jare2", "kaki1", "kaki2"],            frameMs: 220 },
        "run_panic": { frames: ["dwleft1", "dwleft2"],                 frameMs: 120 },
        "scratch":   { frames: ["ltogi1", "ltogi2"],                   frameMs: 200 }
    })

    property int frameIndex: 0
    property string lastAnimation: ""

    implicitWidth: PlasmaCore.Units.iconSizes.medium
    implicitHeight: PlasmaCore.Units.iconSizes.medium

    Image {
        id: sprite
        anchors.fill: parent
        anchors.margins: 2
        source: {
            const a = compact.animations[compact.animation] || compact.animations["sit_calm"];
            const frame = a.frames[compact.frameIndex % a.frames.length];
            return compact.spriteDir + frame + ".png";
        }
        // Pixel art: never smooth.
        smooth: false
        mipmap: false
        fillMode: Image.PreserveAspectFit
        visible: false  // hidden; we render through ColorOverlay below
    }

    ColorOverlay {
        anchors.fill: sprite
        source: sprite
        // Use Kirigami's text colour, which maps to the panel theme's
        // foreground; reads on both Breeze Light and Breeze Dark.
        color: Kirigami.Theme.textColor
    }

    Timer {
        id: frameTimer
        interval: {
            const a = compact.animations[compact.animation] || compact.animations["sit_calm"];
            return a.frameMs;
        }
        running: true
        repeat: true
        onTriggered: {
            const a = compact.animations[compact.animation] || compact.animations["sit_calm"];
            compact.frameIndex = (compact.frameIndex + 1) % a.frames.length;
        }
    }

    onAnimationChanged: {
        if (animation !== lastAnimation) {
            lastAnimation = animation;
            frameIndex = 0;
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton
        onClicked: Plasmoid.expanded = !Plasmoid.expanded
    }
}
