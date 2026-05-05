// What's shown in the panel slot: the active emoji at panel size.
//
// In Plasma 6 this representation is loaded by the panel; clicking it expands
// to the full representation via PlasmoidItem.expanded (handled in main.qml).

import QtQuick
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.plasmoid

Item {
    id: compact

    property string emoji: "🐈"
    property string headline: ""

    implicitWidth: PlasmaCore.Units.iconSizes.medium
    implicitHeight: PlasmaCore.Units.iconSizes.medium

    Text {
        id: emojiText
        anchors.centerIn: parent
        text: compact.emoji
        font.pixelSize: parent.height * 0.85
        font.family: "Noto Color Emoji, Twemoji Mozilla, Apple Color Emoji"
        renderType: Text.NativeRendering
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter

        Behavior on text {
            SequentialAnimation {
                NumberAnimation { target: emojiText; property: "opacity"; to: 0.4; duration: 120 }
                PropertyAction  { target: emojiText; property: "text" }
                NumberAnimation { target: emojiText; property: "opacity"; to: 1.0; duration: 180 }
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton
        onClicked: Plasmoid.expanded = !Plasmoid.expanded
    }
}
