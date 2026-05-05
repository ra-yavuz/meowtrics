// What's shown in the panel: the animated emoji at panel size.

import QtQuick 2.15
import org.kde.plasma.core as PlasmaCore

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

        // Subtle fade on emoji change so frames don't pop.
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
        onClicked: plasmoid.expanded = !plasmoid.expanded
    }
}
