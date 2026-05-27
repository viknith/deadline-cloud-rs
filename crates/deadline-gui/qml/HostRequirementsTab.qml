import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import com.amazon.deadline.gui

ColumnLayout {
    id: hrRoot
    spacing: 12

    required property HostRequirementsModel model

    ButtonGroup { id: hrRadioGroup }

    RadioButton {
        text: "Run on all available worker hosts"
        checked: !hrRoot.model.use_custom_requirements
        ButtonGroup.group: hrRadioGroup
        Accessible.name: "Run on all available worker hosts"
        onCheckedChanged: if (checked) hrRoot.model.use_custom_requirements = false
    }
    RadioButton {
        text: "Run on worker hosts that meet the following requirements"
        checked: hrRoot.model.use_custom_requirements
        ButtonGroup.group: hrRadioGroup
        Accessible.name: "Run on worker hosts that meet the following requirements"
        onCheckedChanged: if (checked) hrRoot.model.use_custom_requirements = true
    }

    // OS / CPU section
    GroupBox {
        Layout.fillWidth: true
        enabled: hrRoot.model.use_custom_requirements
        ColumnLayout {
            anchors.left: parent.left
            anchors.right: parent.right
            RowLayout {
                Label { text: "Operating system"; Layout.minimumWidth: 120 }
                CheckBox { text: "Linux"; checked: hrRoot.model.os_linux; Accessible.name: "Linux"; onCheckedChanged: hrRoot.model.os_linux = checked }
                CheckBox { text: "macOS"; checked: hrRoot.model.os_macos; Accessible.name: "macOS"; onCheckedChanged: hrRoot.model.os_macos = checked }
                CheckBox { text: "Windows"; checked: hrRoot.model.os_windows; Accessible.name: "Windows"; onCheckedChanged: hrRoot.model.os_windows = checked }
            }
            RowLayout {
                Label { text: "CPU architecture"; Layout.minimumWidth: 120 }
                CheckBox { text: "x86_64"; checked: hrRoot.model.cpu_x86_64; Accessible.name: "x86_64"; onCheckedChanged: hrRoot.model.cpu_x86_64 = checked }
                CheckBox { text: "ARM64"; checked: hrRoot.model.cpu_arm64; Accessible.name: "ARM64"; onCheckedChanged: hrRoot.model.cpu_arm64 = checked }
            }
        }
    }

    // Hardware requirements
    GroupBox {
        title: "Hardware requirements"
        Layout.fillWidth: true
        enabled: hrRoot.model.use_custom_requirements
        GridLayout {
            columns: 5
            columnSpacing: 8
            rowSpacing: 8
            anchors.left: parent.left
            anchors.right: parent.right

            Label { text: "vCPUs"; Accessible.name: "vCPUs"; Layout.minimumWidth: 110 }
            Label { text: "Min" }
            SpinBox {
                from: -1; to: 2147483647
                Layout.fillWidth: true
                textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                Accessible.name: "vCPUs min"
                Component.onCompleted: value = hrRoot.model.cpu_min
                onValueChanged: hrRoot.model.cpu_min = value
            }
            Label { text: "Max" }
            SpinBox {
                from: -1; to: 2147483647
                Layout.fillWidth: true
                textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                Accessible.name: "vCPUs max"
                Component.onCompleted: value = hrRoot.model.cpu_max
                onValueChanged: hrRoot.model.cpu_max = value
            }

            Label { text: "Memory (GiB)"; Accessible.name: "Memory (GiB)"; Layout.minimumWidth: 110 }
            Label { text: "Min" }
            SpinBox {
                from: -1; to: 2147483647
                Layout.fillWidth: true
                textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                Accessible.name: "Memory min"
                Component.onCompleted: value = hrRoot.model.memory_gib_min
                onValueChanged: hrRoot.model.memory_gib_min = value
            }
            Label { text: "Max" }
            SpinBox {
                from: -1; to: 2147483647
                Layout.fillWidth: true
                textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                Accessible.name: "Memory max"
                Component.onCompleted: value = hrRoot.model.memory_gib_max
                onValueChanged: hrRoot.model.memory_gib_max = value
            }

            Label { text: "GPUs"; Accessible.name: "GPUs"; Layout.minimumWidth: 110 }
            Label { text: "Min" }
            SpinBox {
                from: -1; to: 2147483647
                Layout.fillWidth: true
                textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                Accessible.name: "GPUs min"
                Component.onCompleted: value = hrRoot.model.gpu_min
                onValueChanged: hrRoot.model.gpu_min = value
            }
            Label { text: "Max" }
            SpinBox {
                from: -1; to: 2147483647
                Layout.fillWidth: true
                textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                Accessible.name: "GPUs max"
                Component.onCompleted: value = hrRoot.model.gpu_max
                onValueChanged: hrRoot.model.gpu_max = value
            }

            Label { text: "GPU memory (GiB)"; Accessible.name: "GPU memory (GiB)"; Layout.minimumWidth: 110 }
            Label { text: "Min" }
            SpinBox {
                from: -1; to: 2147483647
                Layout.fillWidth: true
                textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                Accessible.name: "GPU memory min"
                Component.onCompleted: value = hrRoot.model.gpu_memory_gib_min
                onValueChanged: hrRoot.model.gpu_memory_gib_min = value
            }
            Label { text: "Max" }
            SpinBox {
                from: -1; to: 2147483647
                Layout.fillWidth: true
                textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                Accessible.name: "GPU memory max"
                Component.onCompleted: value = hrRoot.model.gpu_memory_gib_max
                onValueChanged: hrRoot.model.gpu_memory_gib_max = value
            }

            Label { text: "Scratch space (GiB)"; Accessible.name: "Scratch space (GiB)"; Layout.minimumWidth: 110 }
            Label { text: "Min" }
            SpinBox {
                from: -1; to: 2147483647
                Layout.fillWidth: true
                textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                Accessible.name: "Scratch min"
                Component.onCompleted: value = hrRoot.model.scratch_gib_min
                onValueChanged: hrRoot.model.scratch_gib_min = value
            }
            Label { text: "Max" }
            SpinBox {
                from: -1; to: 2147483647
                Layout.fillWidth: true
                textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                Accessible.name: "Scratch max"
                Component.onCompleted: value = hrRoot.model.scratch_gib_max
                onValueChanged: hrRoot.model.scratch_gib_max = value
            }
        }
    }

    // Custom host requirements
    GroupBox {
        title: "Custom host requirements"
        Layout.fillWidth: true
        enabled: hrRoot.model.use_custom_requirements
        ColumnLayout {
            anchors.left: parent.left
            anchors.right: parent.right
            spacing: 6

            // Info toggle
            RowLayout {
                spacing: 4
                Label {
                    text: "ℹ"
                    color: "#5599ff"
                    font.pointSize: 12
                }
                Label {
                    text: "More info"
                    font.pointSize: 10
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: infoText.visible = !infoText.visible
                    }
                }
            }
            Label {
                id: infoText
                visible: false
                text: "Custom worker requirements: With this feature, you can define your own custom worker capabilities.\n\n• Amount — define the quantity of something the worker needs (e.g. number of licenses required).\n• Attribute — define a property the worker needs. Attributes are always a set of strings (e.g. SoftwareConfig = Option1)."
                wrapMode: Text.Wrap
                Layout.fillWidth: true
                font.pointSize: 10
            }

            // Custom amounts
            Repeater {
                id: amountsRepeater
                model: {
                    var raw = hrRoot.model.custom_amounts.toString()
                    if (!raw) return []
                    return raw.split("|").map(function(entry, idx) {
                        var parts = entry.split(";")
                        return { amtIdx: idx, amtName: parts[0] || "", amtMin: parseInt(parts[1]) || -1, amtMax: parseInt(parts[2]) || -1 }
                    })
                }
                delegate: ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    property int myIdx: modelData.amtIdx
                    Rectangle { Layout.fillWidth: true; height: 1; color: palette.mid; visible: myIdx > 0 }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Amount " + (myIdx + 1); font.bold: true }
                        Item { Layout.fillWidth: true }
                        Button {
                            text: "Delete"
                            Accessible.name: "Delete amount " + (myIdx + 1)
                            onClicked: hrRoot.model.remove_custom_amount(myIdx)
                        }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Name"; Layout.minimumWidth: 40 }
                        TextField {
                            id: amtNameField
                            placeholderText: "e.g. licenses.my_app"
                            Layout.fillWidth: true
                            Accessible.name: "Amount name " + (myIdx + 1)
                            Component.onCompleted: text = modelData.amtName
                            onEditingFinished: hrRoot.model.update_custom_amount(myIdx, text, amtMinSpin.value, amtMaxSpin.value)
                        }
                        Label { text: "Min" }
                        SpinBox {
                            id: amtMinSpin
                            from: -1; to: 2147483647
                            Layout.preferredWidth: 100
                            textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                            Accessible.name: "Amount min " + (myIdx + 1)
                            Component.onCompleted: value = modelData.amtMin
                            onValueChanged: hrRoot.model.update_custom_amount(myIdx, amtNameField.text, value, amtMaxSpin.value)
                        }
                        Label { text: "Max" }
                        SpinBox {
                            id: amtMaxSpin
                            from: -1; to: 2147483647
                            Layout.preferredWidth: 100
                            textFromValue: function(value) { return value < 0 ? "" : value.toString() }
                            Accessible.name: "Amount max " + (myIdx + 1)
                            Component.onCompleted: value = modelData.amtMax
                            onValueChanged: hrRoot.model.update_custom_amount(myIdx, amtNameField.text, amtMinSpin.value, value)
                        }
                    }
                }
            }

            // Custom attributes
            Repeater {
                id: attrsRepeater
                model: {
                    var raw = hrRoot.model.custom_attributes.toString()
                    if (!raw) return []
                    return raw.split("|").map(function(entry, idx) {
                        var parts = entry.split(";")
                        return { attrIdx: idx, attrName: parts[0] || "", attrOption: parts[1] || "allOf", attrValues: parts[2] || "" }
                    })
                }
                delegate: ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    property int myIdx: modelData.attrIdx
                    property string myName: modelData.attrName
                    property string myOption: modelData.attrOption
                    property string myValues: modelData.attrValues

                    Rectangle { Layout.fillWidth: true; height: 1; color: palette.mid; visible: myIdx > 0 || amountsRepeater.count > 0 }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Attribute " + (myIdx + 1); font.bold: true }
                        Item { Layout.fillWidth: true }
                        Button {
                            text: "Delete"
                            Accessible.name: "Delete attribute " + (myIdx + 1)
                            onClicked: hrRoot.model.remove_custom_attribute(myIdx)
                        }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "Name"; Layout.minimumWidth: 40 }
                        TextField {
                            id: attrNameField
                            placeholderText: "e.g. software.config"
                            Layout.fillWidth: true
                            Accessible.name: "Attribute name " + (myIdx + 1)
                            Component.onCompleted: text = myName
                            onEditingFinished: {
                                var vals = []
                                for (var i = 0; i < valueRepeater.count; i++) {
                                    var item = valueRepeater.itemAt(i)
                                    if (item) vals.push(item.valueText)
                                }
                                hrRoot.model.update_custom_attribute(myIdx, text, attrAllOf.checked ? "allOf" : "anyOf", vals.join(","))
                            }
                        }
                        RadioButton {
                            id: attrAllOf
                            text: "All"
                            Accessible.name: "All of attribute " + (myIdx + 1)
                            Component.onCompleted: checked = (myOption === "allOf")
                            onCheckedChanged: {
                                if (checked) {
                                    var vals = []
                                    for (var i = 0; i < valueRepeater.count; i++) {
                                        var item = valueRepeater.itemAt(i)
                                        if (item) vals.push(item.valueText)
                                    }
                                    hrRoot.model.update_custom_attribute(myIdx, attrNameField.text, "allOf", vals.join(","))
                                }
                            }
                        }
                        RadioButton {
                            id: attrAnyOf
                            text: "Any"
                            Accessible.name: "Any of attribute " + (myIdx + 1)
                            Component.onCompleted: checked = (myOption !== "allOf")
                            onCheckedChanged: {
                                if (checked) {
                                    var vals = []
                                    for (var i = 0; i < valueRepeater.count; i++) {
                                        var item = valueRepeater.itemAt(i)
                                        if (item) vals.push(item.valueText)
                                    }
                                    hrRoot.model.update_custom_attribute(myIdx, attrNameField.text, "anyOf", vals.join(","))
                                }
                            }
                        }
                    }
                    // Value rows
                    Repeater {
                        id: valueRepeater
                        model: {
                            var v = myValues
                            if (!v) return [""]
                            var arr = v.split(",")
                            return arr.length > 0 ? arr : [""]
                        }
                        delegate: RowLayout {
                            Layout.fillWidth: true
                            Layout.leftMargin: 40
                            property string valueText: valField.text
                            TextField {
                                id: valField
                                placeholderText: "value"
                                Layout.fillWidth: true
                                Accessible.name: "Attribute " + (myIdx + 1) + " value " + (index + 1)
                                Component.onCompleted: text = modelData
                                onEditingFinished: {
                                    var vals = []
                                    for (var i = 0; i < valueRepeater.count; i++) {
                                        var item = valueRepeater.itemAt(i)
                                        if (item) vals.push(item.valueText)
                                    }
                                    hrRoot.model.update_custom_attribute(myIdx, attrNameField.text, attrAllOf.checked ? "allOf" : "anyOf", vals.join(","))
                                }
                            }
                            Button {
                                text: "Remove"
                                enabled: valueRepeater.count > 1
                                Accessible.name: "Remove value " + (index + 1) + " of attribute " + (myIdx + 1)
                                onClicked: {
                                    var allAttrs = hrRoot.model.custom_attributes.toString().split("|")
                                    var parts = allAttrs[myIdx].split(";")
                                    var valueList = parts[2] ? parts[2].split(",") : [""]
                                    valueList.splice(index, 1)
                                    parts[2] = valueList.join(",")
                                    allAttrs[myIdx] = parts.join(";")
                                    hrRoot.model.custom_attributes = allAttrs.join("|")
                                }
                            }
                        }
                    }
                    Button {
                        text: "Add value"
                        Layout.leftMargin: 40
                        Accessible.name: "Add value to attribute " + (myIdx + 1)
                        onClicked: {
                            var allAttrs = hrRoot.model.custom_attributes.toString().split("|")
                            var parts = allAttrs[myIdx].split(";")
                            parts[2] = parts[2] + ","
                            allAttrs[myIdx] = parts.join(";")
                            hrRoot.model.custom_attributes = allAttrs.join("|")
                        }
                    }
                }
            }

            // Add amount / Add attribute buttons (inline)
            RowLayout {
                Layout.fillWidth: true
                spacing: 8
                Button {
                    text: "Add amount"
                    Accessible.name: "Add amount"
                    onClicked: hrRoot.model.add_custom_amount()
                }
                Button {
                    text: "Add attribute"
                    Accessible.name: "Add attribute"
                    onClicked: hrRoot.model.add_custom_attribute()
                }
            }
        }
    }
}
