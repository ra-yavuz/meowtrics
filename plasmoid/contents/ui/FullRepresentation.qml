// The popup shown when the user clicks the panel emoji: large emoji on top,
// a one-line headline, and a per-sensor list with state and last value.

import QtQuick
import QtQuick.Layouts
import org.kde.plasma.components as PC3
import org.kde.plasma.core as PlasmaCore

ColumnLayout {
    id: full
    spacing: PlasmaCore.Units.smallSpacing

    property string activeEmoji: "🐈"
    property string headline: ""
    property var sensors: []

    Layout.preferredWidth: 320
    Layout.preferredHeight: 360

    Text {
        Layout.alignment: Qt.AlignHCenter
        text: full.activeEmoji
        font.pixelSize: 96
        font.family: "Noto Color Emoji, Twemoji Mozilla, Apple Color Emoji"
        renderType: Text.NativeRendering
    }

    PC3.Label {
        Layout.alignment: Qt.AlignHCenter
        Layout.fillWidth: true
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: full.headline
        font.pointSize: 11
    }

    Rectangle {
        Layout.fillWidth: true
        Layout.preferredHeight: 1
        color: Qt.rgba(0, 0, 0, 0.12)
    }

    ListView {
        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true
        model: full.sensors
        spacing: 4

        delegate: RowLayout {
            width: ListView.view.width
            spacing: 8
            PC3.Label { text: modelData.emoji || ""; font.pixelSize: 18; font.family: "Noto Color Emoji" }
            PC3.Label { text: modelData.sensor || ""; Layout.fillWidth: true }
            PC3.Label { text: modelData.state || ""; opacity: 0.7 }
            PC3.Label {
                text: modelData.value !== undefined ? Number(modelData.value).toFixed(1) : ""
                opacity: 0.5
                font.family: "monospace"
            }
        }
    }

    PC3.Label {
        Layout.alignment: Qt.AlignHCenter
        text: "provided AS IS, no warranty"
        opacity: 0.4
        font.pointSize: 8
    }
}
