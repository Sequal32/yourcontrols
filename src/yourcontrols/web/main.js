var connect_button = document.getElementById('connect-button')
var server_button = document.getElementById('server-button')
var settings_button = document.getElementById('settings-button')
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
var port_div = document.getElementById("port-div")

var username = document.getElementById("username-input")
var session_input = document.getElementById("session-input")
var name_input_join = document.getElementById("name-input-join")
var theme_selector = document.getElementById("theme-select")
var streamer_mode = document.getElementById("streamer-mode")
var use_upnp = document.getElementById("use-upnp")
var enable_log = document.getElementById("enable-log")

var update_rate_input = document.getElementById("update-rate-input")
var timeout_input = document.getElementById("timeout-input")

var name_div = document.getElementById("name-div")
var port_div = document.getElementById("port-div")
var server_div = document.getElementById("server-div")

var rectangle_status = document.getElementById("rectangle-status")
// Radios
var session_ip4radio = document.getElementById("session-ip4")
var server_ip4radio = document.getElementById("server-ip4")
var session_ip6radio = document.getElementById("session-ip6")
var server_ip6radio = document.getElementById("server-ip6")
var cloudMethod = document.getElementById("punchthrough-radio")
var directMethod = document.getElementById("direct-radio")
var relayMethod = document.getElementById("relay-radio")

var sessionDiv = document.getElementById("session-div")
var sessionIpRadios = document.getElementById("session-ip-radios")
var joinIpDiv = document.getElementById("join-ip-div")
var joinPortDiv = document.getElementById("join-port-div")
var joinConnectDirect = document.getElementById("join-connect-direct")
var joinConnectCloud = document.getElementById("join-connect-cloud")
var joinIpInput = document.getElementById("join-ip-input")
var joinPortInput = document.getElementById("join-port-input")

// Network
var networkDiv = document.getElementById("network")
var downloadBandwidth = document.getElementById("download-bandwidth")
var downloadRate = document.getElementById("download-rate")
var uploadBandwidth = document.getElementById("upload-bandwidth")
var uploadRate = document.getElementById("upload-rate")
var networkLoss = document.getElementById("network-loss")
var ping = document.getElementById("network-ping")

var forceButton = document.getElementById("force-button")
var externalIp = document.getElementById("external-ip")
var start_observer = document.getElementById("start-observer")

var is_connected = false
var is_client = false
var on_client = true
var has_control = false

var cacheIpInput = ""

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
        document.getElementById("top-right-card").appendChild(networkDiv)
    } else {
        document.getElementById("not_server_running").hidden = visible;
        document.getElementById("top-left-card").appendChild(networkDiv)
    }
    networkDiv.hidden = !visible
    document.getElementById("is_client_server_running").hidden = visible;
    document.getElementById("not_client_server_running").hidden = !visible;
}

function ResetForm() {
    connect_button.updatetext("success", "Connect")
    server_button.updatetext("primary", "Start Server")
}

function FormButtonsDisabled(disabled) {
    connect_button.disabled = disabled
    server_button.disabled = disabled
    settings_button.disabled = disabled
}

function OnConnected() {
    connect_button.updatetext("danger", "Disconnect")
    server_button.updatetext("danger", "Stop Server")
    
    FormButtonsDisabled(false)
    is_connected = true

    port_input_host.disabled = true
    session_input.disabled = true
    session_ip4radio.disabled = true
    server_ip4radio.disabled = true
    session_ip6radio.disabled = true
    server_ip6radio.disabled = true
    cloudMethod.disabled = true
    relayMethod.disabled = true
    directMethod.disabled = true

    joinConnectCloud.disabled = true
    joinConnectDirect.disabled = true
    joinIpInput.disabled = true
    joinPortInput.disabled = true

    if (streamer_mode.checked) {
        externalIp.hidden = true
        joinIpInput.value = joinIpInput.value.split(/\d/).join("X")
    }

    SetStuffVisible(true)
}

function OnDisconnect(text) {
    alert.updatetext("danger", text)
    is_connected = false
    FormButtonsDisabled(false)
    port_input_host.disabled = false

    session_ip4radio.disabled = false
    session_input.disabled = false
    server_ip4radio.disabled = false
    session_ip6radio.disabled = false
    server_ip6radio.disabled = false
    relayMethod.disabled = false
    cloudMethod.disabled = false
    directMethod.disabled = false

    joinConnectCloud.disabled = false
    joinConnectDirect.disabled = false
    joinIpInput.disabled = false
    joinPortInput.disabled = false

    connectionList.clear()

    joinIpInput.value = cacheIpInput
    externalIp.hidden = false
    forceButton.hidden = true

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

function ValidateName(e) {
    return Validate(e, e.value.trim() != "")
}

function ValidateIp(e) {
    return Validate(e, e.value.match(/^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$/gi) || e.value.match(/(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))/gi))
}

function ValidateHostname(e) {
    return Validate(e, e.value.match(/^(([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\-]*[A-Za-z0-9])$/gi))
}

function LoadSettings(newSettings) {
    joinPortInput.value = newSettings.port
    port_input_host.value = newSettings.port
    
    joinIpInput.value = newSettings.ip
    streamer_mode.checked = newSettings.streamer_mode
    
    username.value = newSettings.name
    timeout_input.value = newSettings.conn_timeout
    update_rate_input.value = newSettings.update_rate
    theme_selector.checked = newSettings.ui_dark_theme
    use_upnp.checked = newSettings.use_upnp
    start_observer.checked = newSettings.start_observer
    enable_log.checked = newSettings.enable_log

    setTheme(newSettings.ui_dark_theme)

    settings = newSettings
}

function UpdateMetrics(metrics) {
    downloadBandwidth.textContent = "↓ " + metrics.receiveBandwidth.toFixed(2) + "KB/s"
    downloadRate.textContent = Math.floor(metrics.receivePackets) + " Packets/s"
    uploadBandwidth.textContent = "↑ " + metrics.sentBandwidth.toFixed(2) + " KB/s"
    uploadRate.textContent = Math.floor(metrics.sentPackets) + " Packets/s"
    networkLoss.textContent = (metrics.packetLoss * 100).toFixed(2) + "% Packet loss"
    ping.textContent = metrics.ping.toFixed(0) + "ms"
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
            $("#not_server_running").append(forceButton)
            OnConnected()
            break;
        case "server_fail":
            OnDisconnect("Server failed to start. Reason: " + data["data"])
            break;
        case "client_fail":
            OnDisconnect("Client disconnected. Reason: " + data["data"])
            break;
        case "server":
            is_client = false
            alert.updatetext("success", "Server started! " + data["data"])
            $("#not_user_client").append(forceButton)
            OnConnected()
            break;
        case "host":
            is_client = false;
            forceButton.hidden = false
            $("#not_server_running").append(forceButton)
            break;
        case "error":
            alert.updatetext("danger", data["data"])
            FormButtonsDisabled(false)
            ResetForm()
            break;
        case "control":
            has_control = true
            connectionList.update()
            connectionList.hideStatusText()
            rectangle_status.style.backgroundColor = "cyan"
            forceButton.hidden = true
            break;
        case "lostcontrol":
            has_control = false
            connectionList.update()
            rectangle_status.style.backgroundColor = "red"
            forceButton.hidden = false
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
            forceButton.hidden = true
            break;
        case "stop_observing":
            rectangle_status.style.backgroundColor = "red"
            forceButton.hidden = false
            break;
        // Other client
        case "set_observing":
            connectionList.setObserver(data["data"], true, !is_client)
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
            version_alert_text.innerHTML = "New Version is available " + data["data"]
            break;
        case "update_failed":
            updateFailed()
            break;
        case "config_msg":
            LoadSettings(JSON.parse(data["data"]))
            break;
        case "metrics":
            UpdateMetrics(JSON.parse(data["data"]))
            break;
    }
}

// Init
window.addEventListener("load", function() {
    invoke({"type":"startup"})
})

function setTheme(isDarkTheme) {
    if (isDarkTheme) {
        var elements = document.getElementsByClassName("themed")
        for (var i = 0; i < elements.length; i++) {
            elements[i].classList.add("bg-dark")
            elements[i].classList.add("text-white")
            elements[i].classList.remove("bg-white")
            elements[i].classList.remove("text-black")
        }
    } else {
        var elements = document.getElementsByClassName("themed")
        for (var i = 0; i < elements.length; i++) {
            elements[i].classList.remove("bg-dark")
            elements[i].classList.remove("text-white")
            elements[i].classList.add("bg-white")
            elements[i].classList.add("text-black")
        }
    }
}

function UpdateAircraft(filename) {
    invoke({"type": "loadAircraft", "config_file_name": filename})
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

cloudMethod.addEventListener("change", function() {
    port_div.hidden = true
})

directMethod.addEventListener("change", function() {
    port_div.hidden = false
})

relayMethod.addEventListener("change", function() {
    port_div.hidden = true
})

joinConnectCloud.addEventListener("change", function() {
    sessionDiv.hidden = false
    joinPortDiv.hidden = true
    joinIpDiv.hidden = true
    sessionIpRadios.hidden = true
})

joinConnectDirect.addEventListener("change", function() {
    sessionDiv.hidden = true
    joinPortDiv.hidden = false
    joinIpDiv.hidden = false
    sessionIpRadios.hidden = false
})

joinPortInput.addEventListener("change", function() {
    port_input_host.value = joinPortInput.value
})

port_input_host.addEventListener("change", function() {
    joinPortInput.value = port_input_host.value
})

forceButton.addEventListener("click", function() {
    invoke({"type": "forceTakeControl"})
    forceButton.hidden = true
})

$("#settings-form").submit(function(e) {
    e.preventDefault()

    var newSettings = {}

    newSettings.name = username.value
    newSettings.conn_timeout = ValidateInt(timeout_input) ? parseInt(timeout_input.value) : null
    newSettings.update_rate = ValidateInt(update_rate_input) ? parseInt(update_rate_input.value) : null
    newSettings.ui_dark_theme = theme_selector.checked
    newSettings.streamer_mode = streamer_mode.checked
    newSettings.use_upnp = use_upnp.checked
    newSettings.start_observer = start_observer.checked
    newSettings.enable_log = enable_log.checked

    for (key in newSettings) {
        if (newSettings[key] === null) {return}
        settings[key] = newSettings[key]
    }

    LoadSettings(settings)
    invoke({"type": "updateConfig", "new_config": settings})
})

$("#main-form-host").submit(function(e) {
    e.preventDefault()

    if (is_connected) {invoke({type: "disconnect"}); return}

    // Get radio button
    const method = cloudMethod.checked ? cloudMethod.value : relayMethod.checked ? relayMethod.value : directMethod.checked ? directMethod.value : "";
    const port_ok = method == "cloudServer" ? true : ValidateInt(port_input_host);

    if (!port_ok || !ValidateName(username)) {return}

    FormButtonsDisabled(true)

    UpdateAircraft(aircraftList.value)
    invoke({
        type: "startServer",
        port: parseInt(port_input_host.value) || 0,
        isipv6: server_ip6radio.checked,
        use_upnp: use_upnp.checked,
        username: username.value,
        method: method
    })

})

$("#main-form-join").submit(function(e) {
    e.preventDefault()

    if (is_connected) {invoke({type: "disconnect"}); return}

    let validname = ValidateName(username)

    if (!validname) {return}

    let method = joinConnectCloud.checked ? joinConnectCloud.value : joinConnectDirect.checked ? joinConnectDirect.value : "";

    let data = {
        type: "connect", 
        session_id: session_input.value.toUpperCase(),
        username: username.value,
        method: method,
        isipv6: session_ip6radio.checked
    }

    cacheIpInput = joinIpInput.value

    if (joinConnectDirect.checked) {
        if (ValidateIp(joinIpInput)) {
            data["ip"] = joinIpInput.value
        } else if (ValidateHostname(joinIpInput)) {
            data["hostname"] = joinIpInput.value
        } else {
            return
        }

        if (!ValidateInt(joinPortInput)) {return}
        
        data["port"] = parseInt(joinPortInput.value)
    } 

    FormButtonsDisabled(true)
    invoke(data);
})

function update() {
    invoke({type:"runUpdater"})
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

$(function () {
    $('[data-toggle="tooltip"]').tooltip()
})

function httpGet()
{
    var xmlHttp = new XMLHttpRequest();
    xmlHttp.open( "GET", "https://api.ipify.org" );
    xmlHttp.send();

    function failed() {
        externalIp.innerHTML = "Your IP: Failed to retrieve"
    }

    xmlHttp.onload = function() {
        externalIp.innerHTML = "Your IP: " + xmlHttp.response
    }
    xmlHttp.onerror = failed
    xmlHttp.ontimeout = failed
}

httpGet()