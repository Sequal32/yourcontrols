var emulatorEnabled = false;
var emulatorConnected = false;

var emulatorPanel = document.getElementById("emulator-panel");
var emulatorSelect = document.getElementById("emulator-var-select");
var emulatorAddButton = document.getElementById("emulator-add-var");
var emulatorRows = document.getElementById("emulator-rows");
var emulatorTable = document.getElementById("emulator-table");
var emulatorEmpty = document.getElementById("emulator-empty");

var emulatorVars = {};
var emulatorRowMap = {};

function EmulatorSetEnabled(enabled) {
    emulatorEnabled = enabled;
    EmulatorUpdateVisibility();
}

function EmulatorSetConnectionState(connected, isClient) {
    emulatorConnected = connected;
    EmulatorUpdateVisibility();
    if (!connected) {
        emulatorVars = {};
        emulatorRowMap = {};
        emulatorRows.innerHTML = "";
        emulatorSelect.innerHTML = "";
        emulatorEmpty.textContent = "No variables added.";
        EmulatorShowTable();
        return;
    }
    if (emulatorEnabled && emulatorConnected) {
        EmulatorRequestVars();
    }
}

function EmulatorUpdateVisibility() {
    if (!emulatorPanel) {
        return;
    }

    emulatorPanel.hidden = !(emulatorEnabled && emulatorConnected);
}

function EmulatorRequestVars() {
    invoke({
        type: "emulatorRequestVars",
    });
}

function EmulatorPopulateVars(vars) {
    emulatorVars = {};
    emulatorSelect.innerHTML = "";

    vars.forEach(function (item) {
        var id = item.id || item.name;
        var displayName = item.display_name || item.name;
        emulatorVars[id] = {
            id: id,
            name: item.name,
            varType: item.var_type,
            value: item.value,
        };

        var option = document.createElement("option");
        option.value = id;
        option.textContent = displayName;
        emulatorSelect.appendChild(option);
    });

    emulatorAddButton.disabled = vars.length === 0;
}

function EmulatorShowTable() {
    if (Object.keys(emulatorRowMap).length === 0) {
        emulatorEmpty.hidden = false;
        emulatorTable.hidden = true;
        return;
    }

    emulatorEmpty.hidden = true;
    emulatorTable.hidden = false;
}

function EmulatorNormalizeValue(varType, value) {
    if (isNaN(value)) {
        return null;
    }

    switch (varType) {
        case "i32":
        case "i64":
            return Math.round(value);
        case "bool":
            return value ? 1 : 0;
        default:
            return value;
    }
}

function EmulatorAddRow(id, name, varType, value) {
    if (emulatorRowMap[id]) {
        EmulatorUpdateRowValue(id, value);
        return;
    }

    var row = document.createElement("tr");
    row.className = "emulator-row";

    var nameCell = document.createElement("td");
    nameCell.textContent = name;

    var typeCell = document.createElement("td");
    var typeBadge = document.createElement("span");
    typeBadge.className = "badge badge-secondary emulator-type";
    typeBadge.textContent = varType;
    typeCell.appendChild(typeBadge);

    var valueCell = document.createElement("td");
    var valueInput = document.createElement("input");
    valueInput.type = "number";
    valueInput.step = varType === "f64" ? "any" : "1";
    if (varType === "bool") {
        valueInput.min = "0";
        valueInput.max = "1";
    }
    valueInput.className = "form-control form-control-sm themed emulator-value-input";
    var normalizedInitial = EmulatorNormalizeValue(varType, value != null ? value : 0.0);
    valueInput.value = normalizedInitial != null ? normalizedInitial : 0.0;
    valueInput.addEventListener("change", function () {
        var normalized = EmulatorNormalizeValue(varType, parseFloat(valueInput.value));
        if (normalized == null) {
            return;
        }
        valueInput.value = normalized;
        EmulatorSendValue(id, normalized);
    });
    valueCell.appendChild(valueInput);

    var chipCell = document.createElement("td");
    chipCell.className = "emulator-chip-cell";

    var actionCell = document.createElement("td");
    var removeButton = document.createElement("button");
    removeButton.type = "button";
    removeButton.className = "btn btn-outline-danger btn-sm emulator-remove";
    removeButton.textContent = "Remove";
    removeButton.addEventListener("click", function () {
        if (!emulatorRowMap[id]) {
            return;
        }

        emulatorRows.removeChild(row);
        delete emulatorRowMap[id];
        EmulatorShowTable();

        invoke({
            type: "emulatorRemoveVar",
            name: id,
        });
    });
    actionCell.appendChild(removeButton);

    function addChip(label, handler) {
        var chip = document.createElement("button");
        chip.type = "button";
        chip.className = "btn btn-outline-secondary btn-sm emulator-chip";
        chip.textContent = label;
        chip.addEventListener("click", handler);
        chipCell.appendChild(chip);
    }

    addChip("Toggle", function () {
        var current = parseFloat(valueInput.value) || 0;
        var next = current > 0 ? 0 : 1;
        valueInput.value = next;
        EmulatorSendValue(id, next);
    });

    [1, 100, 1000].forEach(function (amount) {
        addChip("+" + amount, function () {
            var current = parseFloat(valueInput.value) || 0;
            var next = EmulatorNormalizeValue(varType, current + amount);
            if (next == null) {
                return;
            }
            valueInput.value = next;
            EmulatorSendValue(id, next);
        });
    });

    [-1, -100, -1000].forEach(function (amount) {
        addChip(amount.toString(), function () {
            var current = parseFloat(valueInput.value) || 0;
            var next = EmulatorNormalizeValue(varType, current + amount);
            if (next == null) {
                return;
            }
            valueInput.value = next;
            EmulatorSendValue(id, next);
        });
    });

    row.appendChild(nameCell);
    row.appendChild(typeCell);
    row.appendChild(valueCell);
    row.appendChild(chipCell);
    row.appendChild(actionCell);

    emulatorRows.appendChild(row);
    emulatorRowMap[id] = {
        row: row,
        input: valueInput,
        varType: varType,
    };

    EmulatorShowTable();
}

function EmulatorUpdateRowValue(id, value) {
    var rowInfo = emulatorRowMap[id];
    if (!rowInfo) {
        return;
    }

    var normalized = EmulatorNormalizeValue(rowInfo.varType, value != null ? value : 0.0);
    rowInfo.input.value = normalized != null ? normalized : 0.0;
}

function EmulatorSendValue(id, value) {
    if (isNaN(value)) {
        return;
    }

    emulatorVars[id] = emulatorVars[id] || { varType: "f64", value: value };
    var varType = emulatorVars[id].varType || "f64";
    var normalized = EmulatorNormalizeValue(varType, value);
    if (normalized == null) {
        return;
    }

    emulatorVars[id].value = normalized;

    invoke({
        type: "emulatorSetVar",
        name: id,
        value: normalized,
    });
}

function EmulatorHandleVarValue(info) {
    if (!info) {
        return;
    }

    var id = info.id || info.name;
    emulatorVars[id] = {
        id: id,
        varType: info.var_type,
        value: info.value,
    };

    EmulatorAddRow(id, info.name, info.var_type, info.value);
}

function EmulatorMessageReceived(data) {
    switch (data["type"]) {
        case "emulator_enabled":
            EmulatorSetEnabled(data["data"] === "true");
            break;
        case "emulator_vars":
            EmulatorPopulateVars(JSON.parse(data["data"]));
            break;
        case "emulator_value":
            EmulatorHandleVarValue(JSON.parse(data["data"]));
            break;
        case "emulator_error":
            emulatorEmpty.textContent = data["data"];
            emulatorEmpty.hidden = false;
            break;
    }
}

if (emulatorAddButton) {
    emulatorAddButton.addEventListener("click", function () {
        var selected = emulatorSelect.value;
        if (!selected) {
            return;
        }

        invoke({
            type: "emulatorAddVar",
            name: selected,
        });
    });
}
