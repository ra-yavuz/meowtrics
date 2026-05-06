// Custom tooltip shown when hovering the meowtrics panel widget.
// Big animated cat + the catchy random headline + per-sensor table.

import QtQuick
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects
import org.kde.plasma.components as PC3
import org.kde.kirigami as Kirigami

ColumnLayout {
    id: tip
    spacing: Kirigami.Units.smallSpacing

    property string animation: "sit_calm"
    property string headline: ""
    property var sensors: []

    Layout.preferredWidth: 320
    Layout.preferredHeight: 280

    // Big animated cat (re-uses the panel sprite engine).
    Item {
        Layout.alignment: Qt.AlignHCenter
        Layout.preferredWidth: 96
        Layout.preferredHeight: 96
        Layout.topMargin: Kirigami.Units.smallSpacing
        CompactRepresentation {
            anchors.fill: parent
            animation: tip.animation
        }
    }

    PC3.Label {
        Layout.alignment: Qt.AlignHCenter
        Layout.fillWidth: true
        Layout.leftMargin: Kirigami.Units.smallSpacing
        Layout.rightMargin: Kirigami.Units.smallSpacing
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: tip.headline
        font.pointSize: 10
    }

    Rectangle {
        Layout.fillWidth: true
        Layout.preferredHeight: 1
        color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.18)
    }

    ListView {
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.leftMargin: Kirigami.Units.smallSpacing
        Layout.rightMargin: Kirigami.Units.smallSpacing
        Layout.bottomMargin: Kirigami.Units.smallSpacing
        clip: true
        model: tip.sensors
        spacing: 2

        delegate: RowLayout {
            width: ListView.view.width
            spacing: 8
            PC3.Label { text: modelData.emoji || ""; font.pixelSize: 16; font.family: "Noto Color Emoji" }
            PC3.Label { text: modelData.sensor || ""; Layout.fillWidth: true; font.pointSize: 9 }
            PC3.Label { text: modelData.state || ""; opacity: 0.7; font.pointSize: 9 }
            PC3.Label {
                text: modelData.value !== undefined ? Number(modelData.value).toFixed(1) : ""
                opacity: 0.5
                font.family: "monospace"
                font.pointSize: 9
            }
        }
    }
}
