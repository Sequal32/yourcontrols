var connect_button = document.getElementById('connect-button')
var server_button = document.getElementById('server-button')
var alert = document.getElementById("alert")
var version_alert_text = document.getElementById("version-alert-text")
var overloaded_alert = document.getElementById("overloaded-alert")

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

var mediaQueryList = window.matchMedia('(prefers-color-scheme: dark)');

function themeChange (event) {
    if (event.matches) {
        document.body.classList.add(["bg-dark","text-white"])
        document.body.classList.remove(["bg-white","text-dark"])
    } else {
        document.body.classList.remove(["bg-dark","text-white"])
        document.body.classList.add(["bg-white","text-dark"])
    }
};
mediaQueryList.addListener(themeChange)

// Connection List
var connectionList = new ConnectionList(document.getElementById("connection-list"))
// General functions
function invoke(data) {
    window.external.invoke(JSON.stringify(data))
}

function ResetForm() {
    server_input.classList.remove(["is-valid", "is-invalid"])
    port_input.classList.remove(["is-valid", "is-invalid"])
    connect_button.updatetext("success", "Connect")
    server_button.updatetext("primary", "Start Server")
}

function Validate(e, isValid) {
    e.classList.add(isValid ? "is-valid" : "is-invalid")
    e.classList.remove(isValid ? "is-invalid" : "is-valid")
}

function OnConnected() {
    PagesVisible(false)
    connect_button.updatetext("danger", "Disconnect")
    server_button.updatetext("danger", "Stop Server")
    
    trying_connection = false
    server_input.disabled = true
    port_input.disabled = true
    name_input.disabled = true
    ip4radio.disabled = true
    ip6radio.disabled = true
    is_connected = true

    version_alert.hidden = true
}

function OnDisconnect(text) {
    alert.updatetext("danger", text)
    is_connected = false
    trying_connection = false
    server_input.disabled = false
    port_input.disabled = false
    name_input.disabled = false

    ip4radio.disabled = false
    ip6radio.disabled = false

    connectionList.clear()

    PagesVisible(true)
    ResetForm()
}

function ServerClientPageChange(isClient) {
    server_client_page.hidden = false
    on_client = isClient
    connect_button.disabled = !isClient
    server_button.disabled = isClient
}


function PagesVisible(visible) {
    if (visible) {
        server_page_button.hidden = false
        client_page_button.hidden = false
        aircraft_list_button.hidden = false
    } else {
        if (on_client) {
            server_page_button.hidden = true
        } else {
            client_page_button.hidden = true
        }
        aircraft_list_button.hidden = true
    }
}

function ValidatePort(str) {
    return str.match(/\d+/gi)
}

function ValidateIp(str) {
    return str.match(/^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$/gi) || str.match(/(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))/gi)
}

function ValidateHostname(str) {
    return str.match(/^(([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\-]*[A-Za-z0-9])$/gi)
}

// Handle server messages
function MessageReceived(data) {
    switch (data["type"]) {
        case "attempt":
            alert.updatetext("warning", "Attempting connection...")
            break;
        case "connected":
            OnConnected()
            is_client = true
            alert.updatetext("success", "Connected to server.")
            connect_button.updatetext("danger", "Disconnect")
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
            OnConnected()
            is_client = false
            alert.updatetext("success", "Server started! " + data["data"] + " clients connected.")
            connectionList.noPlayers();
            break;
        case "error":
            alert.updatetext("danger", data["data"])
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
        case "set_ip":
            server_input_join.value = data["data"]
            break;
        case "set_port":
            port_input_host.value = data["data"]
            port_input_join.value = data["data"]
            break;
        case "set_name":
            username.value = data["data"]
            break;
        case "overloaded":
            overloaded_alert.hidden = false
            break;
        case "stable":
            overloaded_alert.hidden = true
            break;
        case "newconnection":
            connectionList.add(data["data"])
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
        case "select_active_config":
            aircraftList.searchSelectActive(data["data"])
            break;
        case "version":
            $('#updateModal').modal()
            version_alert_text.innerHTML = `New Version is available ${data["data"]}`
            break;
        case "update_failed":
            updateFailed()
            break;
        case "theme_update":
            if (data["data"] == "true") {
                var elements = document.getElementsByClassName("themed")
                for (const element of elements) {
                    element.classList.add("bg-dark")
                    element.classList.add("text-white")
                    element.classList.remove("bg-white")
                    element.classList.remove("text-black")
                }
            } else if (data["data"] == "false") {
                var elements = document.getElementsByClassName("themed")
                for (const element of elements) {
                    element.classList.remove("bg-dark")
                    element.classList.remove("text-white")
                    element.classList.add("bg-white")
                    element.classList.add("text-black")
                }
            }
            break;
        case "config_msg":
            var json = JSON.parse(data["data"])
            server_input_join.value = json.ip
            port_input_join.value = json.ip
            port_input_host.value = json.port
            port_input_join.value = json.port
            username.value = json.name
            aircraftList.value = json.last_config
            buffer_input.value = json.buffer_size
            timeout_input.value = json.conn_timeout
            update_rate_input.value = json.update_rate
            break;
    }
}

invoke({"type":"startup"})
connectionList.notRunning();

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


document.getElementById("settings-form").onsubmit = function(e) {
    e.preventDefault()

    var settings = {}

    settings.ip = server_input_join.value
    settings.name = username.value
    settings.last_config = aircraftList.value
    settings.buffer_size = buffer_input.value
    settings.conn_timeout = timeout_input.value
    settings.update_rate = update_rate_input.value

    $('#restartModal').modal()

    invoke({"type": "SaveConfig", "Config": settings})

}

document.getElementById("main-form-host").onsubmit = function(e) {
    e.preventDefault()

    if (!validport || !validname) {return}
    trying_connection = true
    invoke({type: "server", port: parseInt(port_input_join.value), is_v6: ip6radio.checked, username: username.value})

}

document.getElementById("main-form-join").onsubmit = function(e) {
    e.preventDefault()

    if (trying_connection) {return}
    if (is_connected) {invoke({type: "disconnect"}); return}

    var validip = ValidateIp(server_input_join.value)
    var validhostname = ValidateHostname(server_input_join.value)
    var validport = ValidatePort(port_input_join.value)
    let validname = name_input.value.trim() != ""

    Validate(port_input_join, validport)
    Validate(server_input_join, validname)

    let data = {type: "connect", port: parseInt(port_input.value)}

    Validate(server_input, validip || validhostname)

    if (!validname || !validport || (!validip && !validhostname)) {return}
    // Match hostname or ip
    if (validhostname) {
        data["hostname"] = server_input.value
    } else if (validip) {
        data["ip"] = server_input.value
    }
    else {
        return
    }
    data["username"] = name_input.value
    trying_connection = true
    invoke(data);
}

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