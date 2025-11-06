import QtQuick 2.15
import QtQuick.Controls 2.15

Dialog {
    id: myDialog
    title: "Hello from Rust!"
    modal: true
    standardButtons: Dialog.Ok | Dialog.Cancel

    contentItem: Text {
        text: "This dialog is controlled by Rust via qmetaobject."
        wrapMode: Text.WordWrap
        width: parent.width
    }

    onAccepted: backend.dialogAccepted()
    onRejected: backend.dialogRejected()
}
