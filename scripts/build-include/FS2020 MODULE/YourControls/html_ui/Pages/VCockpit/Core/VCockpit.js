class VCockpitInstrumentData {
}
class VCockpitPanelData {
}
var globalPanelData = null;
var globalInstrumentListener = RegisterViewListener("JS_LISTENER_INSTRUMENTS");
class VCockpitPanel extends HTMLElement {
    constructor() {
        super(...arguments);
        this.data = null;
        this.curInstrumentIndex = -1;
        this.curAttributes = null;
        this.virtualMouse = null;
        this.vignettage = null;
        this.vignettageNeeded = false;
        this.vignettageHandler = null;
    }
    connectedCallback() {
        if (globalPanelData) {
            this.load(globalPanelData);
        }
        let debugMouse = document.querySelector("#debugmouse");
        if (debugMouse) {
            diffAndSetStyle(debugMouse, StyleProperty.display, "block");
            window.document.addEventListener("mousemove", (e) => {
                debugMouse.style.left = (e.clientX - 7.5) + "px";
                debugMouse.style.top = (e.clientY - 7.5) + "px";
            });
        }
        this.virtualMouse = document.querySelector("#virtualmouse");
        if (this.virtualMouse) {
            window.document.addEventListener("mousemove", (e) => {
                this.virtualMouse.style.left = (e.clientX - 7.5) + "px";
                this.virtualMouse.style.top = (e.clientY - 2.5) + "px";
            });
        }
        this.vignettage = document.querySelector("#vignettage");
    }
    disconnectedCallback() {
    }
    load(_data) {
        this.data = _data;
        this.curInstrumentIndex = -1;
        if (this.data) {
            document.title = _data.sName;
            this.setAttributes(this.data.daAttributes);
            this.loadNextInstrument();
        }
    }
    hasData() {
        return this.data != null;
    }
    setAttributes(_attributes) {
        if (this.curAttributes) {
            for (var i = 0; i < this.curAttributes.length; i++) {
                document.body.removeAttribute(this.curAttributes[i].name);
            }
        }
        this.curAttributes = _attributes;
        let gamepad = false;
        for (var i = 0; i < _attributes.length; i++) {
            diffAndSetAttribute(document.body, _attributes[i].name, _attributes[i].value);
            if (_attributes[i].name == "quality") {
                if (_attributes[i].value == "hidden" || _attributes[i].value == "disabled") {
                    diffAndSetStyle(this, StyleProperty.display, "none");
                }
                else {
                    diffAndSetStyle(this, StyleProperty.display, "block");
                }
            }
            if (_attributes[i].name == "gamepad") {
                if (_attributes[i].value == "true") {
                    gamepad = true;
                }
            }
        }
        if (this.vignettage && gamepad != this.vignettageNeeded) {
            this.vignettageNeeded = gamepad;
            if (!gamepad) {
                this.hideVignettage();
            }
        }
        window.document.dispatchEvent(new Event('OnVCockpitPanelAttributesChanged'));
    }
    showVirtualMouse(_target, _show) {
        if (this.virtualMouse) {
            for (var i = 0; i < this.children.length; i++) {
                var instrument = this.children[i];
                if (instrument) {
                    if (_target && instrument.getAttribute("Guid") != _target)
                        continue;
                    diffAndSetStyle(this.virtualMouse, StyleProperty.display, (_show) ? "block" : "none");
                }
            }
        }
    }
    registerInstrument(_instrumentName, _instrumentClass) {
        var pattern = Include.absolutePath(window.location.pathname, VCockpitPanel.instrumentRoot);
        var stillLoading = Include.isLoadingScript(pattern);
        if (stillLoading) {
            console.log("Still Loading Dependencies. Retrying...");
            setTimeout(this.registerInstrument.bind(this, _instrumentName, _instrumentClass), 1000);
            return;
        }
        window.customElements.define(_instrumentName, _instrumentClass);
        console.log("Instrument registered");
        console.log("Creating instrument " + _instrumentName + "...");
        this.createInstrument(_instrumentName, _instrumentClass);
    }
    createInstrument(_instrumentName, _instrumentClass) {
        try {
            var template = document.createElement(_instrumentName);
        }
        catch (error) {
            console.error("Error while creating instrument. Retrying...");
            setTimeout(this.createInstrument.bind(this, _instrumentName, _instrumentClass), 1000);
            return;
        }
        if (template) {
            console.log("Instrument created");
            this.setupInstrument(template);
            if (this.vignettage) {
                template.addEventListener("mouseenter", (e) => {
                    this.showVignettage(template);
                });
                template.addEventListener("mouseleave", (e) => {
                    this.hideVignettage();
                });
            }
            this.data.daInstruments[this.curInstrumentIndex].templateName = _instrumentName;
            this.data.daInstruments[this.curInstrumentIndex].templateClass = _instrumentClass;
            document.title += " - " + template.instrumentIdentifier;
            // YourControls
            handler = new YourControlsHandler(template.instrumentIdentifier, template.isInteractive)
        }
        this.loadNextInstrument();
    }
    loadNextInstrument() {
        this.curInstrumentIndex++;
        if (this.curInstrumentIndex < this.data.daInstruments.length) {
            var instrument = this.data.daInstruments[this.curInstrumentIndex];
            var url = VCockpitPanel.instrumentRoot + instrument.sUrl;
            console.log("Importing instrument " + url);
            var index = this.urlAlreadyImported(instrument.sUrl);
            if (index >= 0) {
                var instrumentName = this.data.daInstruments[index].templateName;
                var instrumentClass = this.data.daInstruments[index].templateClass;
                console.log("Instrument " + url + " already imported. Creating right now.");
                this.createInstrument(instrumentName, instrumentClass);
            }
            else {
                Include.setAsyncLoading(false);
                Include.addImport(url);
            }
        }
    }
    setupInstrument(_elem) {
        var instrument = this.data.daInstruments[this.curInstrumentIndex];
        var url = VCockpitPanel.instrumentRoot + instrument.sUrl;
        url = Include.absoluteURL(window.location.pathname, url);
        diffAndSetAttribute(_elem, "Guid", instrument.iGUId + '');
        diffAndSetAttribute(_elem, "Url", url);
        var fRatioX = this.data.vDisplaySize.x / this.data.vLogicalSize.x;
        var fRatioY = this.data.vDisplaySize.y / this.data.vLogicalSize.y;
        var x = Math.round(instrument.vPosAndSize.x * fRatioX);
        var y = Math.round(instrument.vPosAndSize.y * fRatioY);
        var w = Math.round(instrument.vPosAndSize.z * fRatioX);
        var h = Math.round(instrument.vPosAndSize.w * fRatioY);
        if (w <= 0)
            w = 10;
        if (h <= 0)
            h = 10;
        _elem.style.position = "absolute";
        _elem.style.left = x + "px";
        _elem.style.top = y + "px";
        _elem.style.width = w + "px";
        _elem.style.height = h + "px";
        _elem.setConfigFile(this.data.sConfigFile);
        this.appendChild(_elem);
    }
    urlAlreadyImported(_url) {
        var realUrl = _url.split("?")[0];
        for (var i = 0; i < this.curInstrumentIndex; i++) {
            var instrumentRealUrl = this.data.daInstruments[i].sUrl.split("?")[0];
            if (realUrl === instrumentRealUrl) {
                return i;
            }
        }
        return -1;
    }
    showVignettage(_elem) {
        if (this.vignettage && this.vignettageNeeded) {
            diffAndSetStyle(this.vignettage, StyleProperty.display, "block");
            this.vignettage.style.top = _elem.clientTop + "px";
            this.vignettage.style.left = _elem.clientLeft + "px";
            this.vignettage.style.width = _elem.clientWidth + "px";
            this.vignettage.style.height = _elem.clientHeight + "px";
            if (this.vignettageHandler != null) {
                let opacity = 0;
                let animSpeed = 2;
                let deltaTime = 0.032;
                this.vignettageHandler = setInterval(() => {
                    if (this.vignettage.style.display == "block") {
                        this.vignettage.style.opacity = opacity + '';
                        opacity += deltaTime * animSpeed;
                        if (opacity > 1.0) {
                            opacity = 1.0;
                            animSpeed = -animSpeed;
                        }
                        else if (opacity < 0.25) {
                            opacity = 0.25;
                            animSpeed = -animSpeed;
                        }
                    }
                }, deltaTime);
            }
        }
    }
    hideVignettage() {
        if (this.vignettage) {
            diffAndSetStyle(this.vignettage, StyleProperty.display, "none");
            if (this.vignettageHandler) {
                clearInterval(this.vignettageHandler);
                this.vignettageHandler = undefined;
            }
        }
    }
}
VCockpitPanel.instrumentRoot = "../Instruments/";
window.customElements.define("vcockpit-panel", VCockpitPanel);
function registerInstrument(_instrumentName, _instrumentClass) {
    var panel = window.document.getElementById("panel");
    if (panel) {
        console.log("Registering instrument " + _instrumentName + "...");
        setTimeout(panel.registerInstrument.bind(panel, _instrumentName, _instrumentClass), 1000);
    }
}
Coherent.on("ShowVCockpitPanel", function (_data) {
    console.log("Initializing Panel " + _data.sName);
    globalPanelData = _data;
    var panel = window.document.getElementById("panel");
    if (panel) {
        if (panel.hasData()) {
            console.log("Reloading panel...");
            window.location.reload(true);
        }
        else {
            panel.load(_data);
        }
    }
});
Coherent.on("RefreshVCockpitPanel", function (_data) {
    var panel = window.document.getElementById("panel");
    if (panel) {
        panel.setAttributes(_data.daAttributes);
    }
});
Coherent.on("OnInteractionEvent", function (_target, _args) {
    if (!closed) {
        var panel = window.document.getElementById("panel");
        if (panel) {
            // YourControls - Returns whether to send data to instruments
            if (!handler.processInteractionEvent(_args)) {
                return
            }
            for (var i = 0; i < panel.children.length; i++) {
                var instrument = panel.children[i];
                if (instrument) {
                    if (_target && instrument.getAttribute("Guid") != _target)
                        continue;
                    instrument.onInteractionEvent(_args);
                }
            }
        }
    }
});
Coherent.on("OnMouseEnter", function (_target, _x, _y) {
    if (!closed && _target) {
        var panel = window.document.getElementById("panel");
        if (panel) {
            for (var i = 0; i < panel.children.length; i++) {
                var instrument = panel.children[i];
                if (instrument) {
                    if (instrument.getAttribute("Guid") != _target)
                        continue;
                    let element = window.document.elementFromPoint(_x, _y);
                    if (element)
                        element.dispatchEvent(new MouseEvent("mouseenter"));
                    instrument.dispatchEvent(new MouseEvent("mouseenter"));
                }
            }
        }
    }
});
Coherent.on("OnMouseLeave", function (_target, _x, _y) {
    if (!closed && _target) {
        var panel = window.document.getElementById("panel");
        if (panel) {
            for (var i = 0; i < panel.children.length; i++) {
                var instrument = panel.children[i];
                if (instrument) {
                    if (instrument.getAttribute("Guid") != _target)
                        continue;
                    let element = window.document.elementFromPoint(_x, _y);
                    if (element)
                        element.dispatchEvent(new MouseEvent("mouseleave"));
                    instrument.dispatchEvent(new MouseEvent("mouseleave"));
                }
            }
        }
    }
});
Coherent.on("ShowVirtualMouse", function (_target, _show) {
    if (!closed) {
        var panel = window.document.getElementById("panel");
        if (panel) {
            panel.showVirtualMouse(_target, _show);
        }
    }
});
Coherent.on("StartHighlight", function (_target, _event) {
    if (!closed) {
        var panel = window.document.getElementById("panel");
        if (panel) {
            for (var i = 0; i < panel.children.length; i++) {
                var instrument = panel.children[i];
                if (instrument) {
                    if (_target && instrument.getAttribute("Guid") != _target)
                        continue;
                    instrument.startHighlight(_event);
                }
            }
        }
    }
});
Coherent.on("StopHighlight", function (_target, _event) {
    if (!closed) {
        var panel = window.document.getElementById("panel");
        if (panel) {
            for (var i = 0; i < panel.children.length; i++) {
                var instrument = panel.children[i];
                if (instrument) {
                    if (_target && instrument.getAttribute("Guid") != _target)
                        continue;
                    instrument.stopHighlight(_event);
                }
            }
        }
    }
});
Coherent.on("OnSoundEnd", function (_target, _eventId) {
    if (!closed) {
        var panel = window.document.getElementById("panel");
        if (panel) {
            for (var i = 0; i < panel.children.length; i++) {
                var instrument = panel.children[i];
                if (instrument) {
                    if (_target && instrument.getAttribute("Guid") != _target)
                        continue;
                    instrument.onSoundEnd(_eventId);
                }
            }
        }
    }
});
Coherent.on("OnAllInstrumentsLoaded", function () {
    if (!closed) {
        BaseInstrument.allInstrumentsLoaded = true;
    }
});
function getDomPath(elt) {
    var path = [];
    while (elt != null) {
        if (elt.id) {
            path.unshift(elt.id);
        }
        else {
            path.unshift(elt.nodeName);
        }
        elt = elt.parentElement;
    }
    return path.join('>');
}
Coherent.on("Raycast", function (_id, _x, _y) {
    var elt = document.elementFromPoint(_x, _y);
    var result = {};
    if (elt && elt.id) {
        result.id = elt.id;
    }
    else {
        result.id = 'none';
    }
    result.path = getDomPath(elt);
    var rect = new DOMRect();
    if (elt) {
        rect = elt.getBoundingClientRect();
    }
    Coherent.trigger('ON_VCOCKPIT_RAYCAST_RESULT', _id, JSON.stringify(result), rect.x, rect.y, rect.width, rect.height);
});
checkAutoload();
if (EDITION_MODE()) {
    g_debugMgr.AddDebugButton("Test VCockpit", () => {
        let data = new VCockpitPanelData();
        data.sName = "VCockpit01";
        data.vLogicalSize = new Vec2(1024, 768);
        data.vDisplaySize = new Vec2(1024, 768);
        data.daAttributes = [];
        data.daInstruments = [];
        let instrument = new VCockpitInstrumentData();
        instrument.iGUId = 0;
        instrument.sUrl = "NavSystems/AS1000/MFD/AS1000_MFD.html";
        instrument.templateName = "";
        instrument.vPosAndSize = { x: 0, y: 0, z: 1024, w: 768 };
        data.daInstruments.push(instrument);
        Coherent.trigger("ShowVCockpitPanel", data);
        setTimeout(() => { Coherent.trigger("OnAllInstrumentsLoaded"); }, 5000);
    });
    let g_UpdateRunning = -1;
    g_debugMgr.AddDebugButton("STOP", () => {
        if (g_UpdateRunning >= 0)
            cancelAnimationFrame(g_UpdateRunning);
    });
    g_debugMgr.AddDebugButton("Test string", () => {
        if (g_UpdateRunning >= 0)
            cancelAnimationFrame(g_UpdateRunning);
        let update = () => {
            for (let i = 0; i < 1000; i++) {
                Coherent.trigger("TEST_EVENT", "HUHU");
            }
            g_UpdateRunning = requestAnimationFrame(update);
        };
        update();
    });
    g_debugMgr.AddDebugButton("Test string variable", () => {
        if (g_UpdateRunning >= 0)
            cancelAnimationFrame(g_UpdateRunning);
        const TEST = "TEST_EVENT";
        const TEST2 = "HUHU";
        let update = () => {
            for (let i = 0; i < 1000; i++) {
                Coherent.trigger(TEST, TEST2);
            }
            g_UpdateRunning = requestAnimationFrame(update);
        };
        update();
    });
    g_debugMgr.AddDebugButton("Test simvar array", () => {
        if (g_UpdateRunning >= 0)
            cancelAnimationFrame(g_UpdateRunning);
        let batches = [];
        for (let i = 0; i < 200; i++) {
            let approachBatch = new SimVar.SimVarBatch("C:fs9gps:FlightPlanApproachWaypointsNumber", "C:fs9gps:FlightPlanWaypointApproachIndex");
            approachBatch.instrumentID = "Instr " + 1;
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachICAO", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachName", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLatitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLongitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegTotalDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachType", "number", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachICAO", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachName", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLatitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLongitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegTotalDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachType", "number", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachICAO", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachName", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLatitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLongitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegTotalDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachType", "number", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachICAO", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachName", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLatitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLongitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegTotalDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachType", "number", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachICAO", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachName", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLatitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLongitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegTotalDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachType", "number", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachICAO", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachName", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLatitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLongitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegTotalDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachType", "number", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachICAO", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachName", "string", "string");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLatitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLongitude", "degree", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachLegTotalDistance", "nautical miles", "number");
            approachBatch.add("C:fs9gps:FlightPlanWaypointApproachType", "number", "number");
            batches.push(approachBatch);
        }
        let update = () => {
            for (let i = 0; i < batches.length; i++) {
                SimVar.GetSimVarArrayValues(batches[i], function (_Values) {
                }, "data_" + i);
            }
            g_UpdateRunning = requestAnimationFrame(update);
        };
        update();
    });
    g_debugMgr.AddDebugButton("Test simvar", () => {
        if (g_UpdateRunning >= 0)
            cancelAnimationFrame(g_UpdateRunning);
        let simName = "TRANSPONDER CODE:1";
        let simUnit = "number";
        let update = () => {
            for (let i = 0; i < 1000; i++) {
                SimVar.GetSimVarValue(simName, simUnit);
            }
            g_UpdateRunning = requestAnimationFrame(update);
        };
        update();
    });
}
// YourControls
Include.addImports([
    "/YourControls/Main.js",
    "/YourControls/Network.js",
    "/YourControls/HTMLTrigger.js",
    "/YourControls/HTMLEvents.js",
]);
//# sourceMappingURL=VCockpit.js.map