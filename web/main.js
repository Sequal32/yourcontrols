var connect_button = document.getElementById('connect-button')
var server_button = document.getElementById('server-button')
var alert = document.getElementById("alert")
var version_alert_text = document.getElementById("version-alert-text")
var overloaded_alert = document.getElementById("overloaded-alert")
var aircraftList = document.getElementById("aircraft-list")

var nav_bar = document.getElementById("nav")
var server_client_page = document.getElementById("server-client-page");
var server_page_button = document.getElementById("server-page")
var client_page_button = document.getElementById("client-page")
var connection_list_button = document.getElementById("connection-page")
var aircraft_list_button = document.getElementById("aircraft-page")

var port_input_host = document.getElementById("port-input-host")

var username = document.getElementById("username-input")
var port_input_join = document.getElementById("port-input-join")
var server_input_join = document.getElementById("server-input-join")
var name_input_join = document.getElementById("name-input-join")
var theme_selector = document.getElementById("theme-select")
var beta_selector = document.getElementById("beta-select")

var update_rate_input = document.getElementById("update-rate-input")
var timeout_input = document.getElementById("timeout-input")
var buffer_input = document.getElementById("buffer-input")

var name_div = document.getElementById("name-div")
var port_div = document.getElementById("port-div")
var server_div = document.getElementById("server-div")

var rectangle_status = document.getElementById("rectangle-status")
// Radios
var radios = document.getElementById("radios")
var ip4radio = document.getElementById("ip4")
var ip6radio = document.getElementById("ip6")

var trying_connection = false
var is_connected = false
var is_client = false
var on_client = true
var has_control = false

var settings = {}

var mediaQueryList = window.matchMedia('(prefers-color-scheme: dark)');

// Connection List
var connectionList = new ConnectionList(document.getElementById("connection-list"))
// General functions
function invoke(data) {
    window.external.invoke(JSON.stringify(data))
}

function SetStuffVisible(visible) {
    if (is_client) {
        document.getElementById("not_user_client").hidden = visible;
        document.getElementById("is_user_client").hidden = !visible;
    } else {
        document.getElementById("not_server_running").hidden = visible;
        document.getElementById("is_server_running").hidden = !visible;
    }
    document.getElementById("is_client_server_running").hidden = visible;
    document.getElementById("not_client_server_running").hidden = !visible;
}

function ResetForm() {
    connect_button.updatetext("success", "Connect")
    server_button.updatetext("primary", "Start Server")
}

function OnConnected() {
    connect_button.updatetext("danger", "Disconnect")
    server_button.updatetext("danger", "Stop Server")
    
    trying_connection = false
    server_input_join.disabled = true
    port_input_join.disabled = true
    port_input_host.disabled = true
    ip4radio.disabled = true
    ip6radio.disabled = true
    is_connected = true

    SetStuffVisible(true)
}

function OnDisconnect(text) {
    alert.updatetext("danger", text)
    is_connected = false
    trying_connection = false
    server_input_join.disabled = false
    port_input_join.disabled = false
    port_input_host.disabled = false

    ip4radio.disabled = false
    ip6radio.disabled = false

    connectionList.clear()

    ResetForm()
    SetStuffVisible(false)
}

function ServerClientPageChange(isClient) {
    server_client_page.hidden = false
    on_client = isClient
    connect_button.disabled = !isClient
    server_button.disabled = isClient
}


function Validate(e, isValid) {
    e.classList.add(isValid ? "is-valid" : "is-invalid")
    e.classList.remove(isValid ? "is-invalid" : "is-valid")
    return isValid
}

function ValidateInt(e) {
    return Validate(e, e.value.match(/\d+/gi))
}

function ValidateIp(e) {
    return Validate(e, e.value.match(/^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$/gi) || e.value.match(/(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))/gi))
}

function ValidateHostname(e) {
    return Validate(e, e.value.match(/^(([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\-]*[A-Za-z0-9])$/gi))
}

function ValidateName(e) {
    return Validate(e, e.value.trim() != "")
}

function LoadSettings(newSettings) {
    server_input_join.value = newSettings.ip
    port_input_join.value = newSettings.ip
    port_input_host.value = newSettings.port
    port_input_join.value = newSettings.port
    username.value = newSettings.name
    aircraftList.value = newSettings.last_config
    buffer_input.value = newSettings.buffer_size
    timeout_input.value = newSettings.conn_timeout
    update_rate_input.value = newSettings.update_rate
    theme_selector.checked = newSettings.ui_dark_theme
    beta_selector.checked = newSettings.check_for_betas

    setTheme(newSettings.ui_dark_theme)

    settings = newSettings
}

// Handle server messages
function MessageReceived(data) {
    switch (data["type"]) {
        case "attempt":
            alert.updatetext("warning", "Attempting connection...")
            break;
        case "connected":
            is_client = true
            alert.updatetext("success", "Connected to server.")
            connect_button.updatetext("danger", "Disconnect")
            document.getElementById("not_user_client").hidden = true;
            document.getElementById("is_user_client").hidden = false;
            OnConnected()
            break;
        case "server_fail":
            OnDisconnect("Server failed to start. Reason: " + data["data"])
            break;
        case "client_fail":
            OnDisconnect("Client disconnected. Reason: " + data["data"])
            break;
        case "disconnected":
            OnDisconnect("Not Connected.")
            break;
        case "server":
            is_client = false
            alert.updatetext("success", "Server started! " + data["data"] + " clients connected.")
            OnConnected()
            break;
        case "error":
            alert.updatetext("danger", data["data"])
            ResetForm()
            break;
        case "control":
            has_control = true
            connectionList.update()
            connectionList.hideStatusText()
            rectangle_status.style.backgroundColor = "cyan"
            break;
        case "lostcontrol":
            has_control = false
            connectionList.update()
            rectangle_status.style.backgroundColor = "red"
            break;
        case "overloaded":
            overloaded_alert.hidden = false
            break;
        case "stable":
            overloaded_alert.hidden = true
            break;
        case "newconnection":
            connectionList.add(data["data"])
            setTheme(settings.ui_dark_theme)
            break;
        case "lostconnection":
            connectionList.remove(data["data"])
            break;
        // Observing
        case "observing":
            rectangle_status.style.backgroundColor = "grey"
            break;
        case "stop_observing":
            rectangle_status.style.backgroundColor = "red"
            break;
        // Other client
        case "set_observing":
            connectionList.setObserver(data["data"], true)
            break;
        case "set_not_observing":
            connectionList.setObserver(data["data"], false)
            break;
        // Other client
        case "set_incontrol":
            connectionList.setInControl(data["data"])
            break;
        // Add possible aircraft
        case "add_aircraft":
            aircraftList.addAircraft(data["data"])
            break;
        case "version":
            $('#updateModal').modal()
            version_alert_text.innerHTML = `New Version is available ${data["data"]}`
            break;
        case "update_failed":
            updateFailed()
            break;
        case "config_msg":
            LoadSettings(JSON.parse(data["data"]))
            break;
    }
}

// Init
invoke({"type":"startup"})

var setTheme = (isDarkTheme) =>{
    if (isDarkTheme) {
        var elements = document.getElementsByClassName("themed")
        for (const element of elements) {
            element.classList.add("bg-dark")
            element.classList.add("text-white")
            element.classList.remove("bg-white")
            element.classList.remove("text-black")
        }
    } else {
        var elements = document.getElementsByClassName("themed")
        for (const element of elements) {
            element.classList.remove("bg-dark")
            element.classList.remove("text-white")
            element.classList.add("bg-white")
            element.classList.add("text-black")
        }
    }
}

function UpdateAircraft(filename) {
    invoke({"type": "load_aircraft", "filename": filename})
}

// Buttons functions

connect_button.updatetext = function(typeString, text) {
    connect_button.className = connect_button.className.replace(/btn-\w+/gi, "btn-" + typeString)
    connect_button.innerHTML = text
}

server_button.updatetext = function(typeString, text) {
    server_button.className = server_button.className.replace(/btn-\w+/gi, "btn-" + typeString)
    server_button.innerHTML = text
}

alert.updatetext = function(typeString, text) {
    alert.className = alert.className.replace(/alert-\w+/gi, "alert-" + typeString)
    // Only change text, do not get rid of rectangle
    alert.childNodes[0].nodeValue = text
}

$("#settings-form").submit(e => {
    e.preventDefault()

    var newSettings = {}

    newSettings.name = username.value
    newSettings.last_config = aircraftList.value
    newSettings.buffer_size = ValidateInt(buffer_input) ? parseInt(buffer_input.value) : null
    newSettings.conn_timeout = ValidateInt(timeout_input) ? parseInt(timeout_input.value) : null
    newSettings.update_rate = ValidateInt(update_rate_input) ? parseInt(update_rate_input.value) : null
    newSettings.ui_dark_theme = theme_selector.checked// == "true"? true : false
    newSettings.check_for_betas = beta_selector.checked // == "true"? true : false

    for (key in newSettings) {
        if (newSettings[key] === null) {return}
        settings[key] = newSettings[key]
    }

    LoadSettings(settings)
    invoke({"type": "SaveConfig", "config": settings})
})

$("#main-form-host").submit(e => {
    e.preventDefault()

    if (trying_connection) {return}
    if (is_connected) {invoke({type: "disconnect"}); return}


    if (!ValidateInt(port_input_host) || !ValidateName(username)) {return}

    trying_connection = true
    UpdateAircraft(aircraftList.value)
    invoke({
        type: "server",
        port: parseInt(port_input_host.value),
        is_v6: ip6radio.checked,
        username: username.value
    })

})

$("#main-form-join").submit(e => {
    e.preventDefault()

    if (trying_connection) {return}
    if (is_connected) {invoke({type: "disconnect"}); return}


    var validip = ValidateIp(server_input_join)
    var validhostname = ValidateHostname(server_input_join)
    var validport = ValidateInt(port_input_join)
    let validname = ValidateName(username)

    let data = {type: "connect", port: parseInt(port_input_join.value)}

    if (!validname || !validport || (!validip && !validhostname)) {return}
    // Match hostname or ip
    if (validhostname) {
        data["hostname"] = server_input_join.value
    } else if (validip) {
        data["ip"] = server_input_join.value
    }

    data["username"] = username.value
    trying_connection = true

    UpdateAircraft(aircraftList.value)
    invoke(data);
})

function update() {
    invoke({type:"update"})
    version_alert_button.classList.add("btn-primary")
    version_alert_button.classList.remove("btn-danger")
    version_alert_button.innerHTML = "Downloading....";
    version_alert_button.disabled = true;
}

function updateFailed() {
    version_alert_button.classList.remove("btn-primary")
    version_alert_button.classList.add("btn-danger")
    version_alert_button.innerHTML = "Failed. Retry?";
    version_alert_button.disabled = false;
}

version_alert_button.onclick = update

aircraftList.addAircraft = function (aircraftName) {
    const newButton = document.createElement("option")
    newButton.className = "list-group-item list-group-item-action aircraft-list-entry themed"
    newButton.innerHTML = aircraftName
    newButton.value = aircraftName

    aircraftList.appendChild(newButton)
}