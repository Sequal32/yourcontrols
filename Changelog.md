# YourControls Changelog

## Version 2.7.7

### Profiles

- Added: Airbus A320neo by LatinVFR (v1.0.2).
- Added: Bell 206L3 by Cowan Simulation (v1.0.2).
- Updated: Airbus A318ceo by LatinVFR to v2.0.0.
- Updated: Airbus A319ceo by LatinVFR to v2.0.0.
- Updated: Airbus A321neo by LatinVFR to v2.0.1.
- Updated: Airbus A32NX by FlyByWire to v0.10.0.
- Updated: Airbus A339X by Headwind Simulations to v0.4.1.
- Updated: Airbus H125 by Cowan Simulation to v1.0.2.
- Updated: Bell 206B3 by Cowan Simulation to v1.0.3.
- Updated: Cessna 414AW Chancellor by Flysimware to v4.3.3.
- Updated: Discus-2c by Got Friends to v2.0.7.
- Updated: Edgley EA-7 by Got Friends to v1.0.7.
- Updated: Grumman G-21A by Microsoft & iniBuilds to v1.1.5.
- Updated: Grumman JRF-6B Goose by Big Radials to v1.0.8.
- Updated: Hughes H4 by Microsoft & Bluemesh to v1.1.8.
- Updated: MD Helicopters 500E by Cowan Simulation to v1.0.2.
- Updated: Piper PA-28 Warrior II by JustFlight to v0.5.6.
- Updated: Piper PA-28R Arrow III by JustFlight to v0.10.6.
- Updated: Piper PA-28R Turbo Arrow by JustFlight to v0.3.6.
- Removed: FlyByWire A32NX Development/Experimental.
- Removed: Pushback tug (conflicts with other add-ons).
- Removed: Support for PMS50 GNS530 (deprecated).

### Aircraft Fixes

- General: Desync when pressing Garmin 330 numbers.
- General: Lots of excess unsettable local var cleanup.
- General: TACAN support updated per latest SDK.
- Asobo/WT Longitude: Backup ADI barometer.
- Microsoft PC-6: Dimmer brightness knobs.
- Microsoft Volocity: Throttle/VS switch.
- Big Radial JRF-6B: Added missing fuel valves.
- Big Radial JRF-6B: Propeller starter animation.
- Cowan H125: Corrected wrong fuel tank variable.
- FSS E175: Updated slider ignores to stop EFB spam.
- PMDG 737: Auto brake selectior knob desync.
- PMDG 737: CPT and FO yoke AP disconnect switches.
- PMDG 737: Disabled CPT and FO minimums selector knobs.
- PMDG 737: Landing and logo light switches w/pulse option.
- PMDG 737: Speed of parking brake set/release.
- SWS RV-10: Garmin G5 power button.

## Version 2.7.6

### Changes

- Added: Boeing 737NG Series by PMDG Simulations (v3.0.65).
- Added: Support for stored radio frequencies.

#### 737 Issues

- Recommend setting unmodified COLDDARK state before sharing.
- Persistent fuel state unsyncable. Simvars being overridden.
- Vertical speed window does not go negative value.
- Altimeter sync is disabled. Sim event is being overridden?
- APU/ENG bus switches may need to be switched twice for effect.
- COM3 does nothing interally. Not using native simvars. Won't sync.
- ADF standby frequency not using native simvar, active does and will sync.
- FLT ALT/LAND ALT knobs currently disabled to prevent desync.

## Version 2.7.5

### Profiles

- Added: Airbus A318ceo by LatinVFR (v1.0.2).
- Added: Airbus H125 by Cowan Simulation (v1.0.0).
- Added: Bell 206B3 by Cowan Simulation (v1.0.2).
- Added: Cessna 172SP Classic by WBSim (v1.0.6).
- Added: Embraer E175 by FlightSim Studio (v0.9.6-experimental).
  - Do not use standby freq/swap. Enter directly into active.
- Added: MD Helicopters 500E by Cowan Simulation (v1.0.1).
- Added: Van's RV-10 by SimWorks Studios (v1.0.3).
- Updated: A32NX (Dev/Exp) by FlyByWire to March 24, 2023.
- Updated: AAU1-Brief updates for Longitude, TBM, and CJ4.
- Updated: Airbus A339X by Headwind Simulations to v0.3.3.
- Updated: Cessna 152 by JPLogistics & WBSim to v2.0.6.
- Updated: Cirrus Vision Jet G2 by FlightFX to v1.2.3.
- Updated: Robinson R44 by RotorSimPilot & Airland to v1.4.
- Readded: Douglas DC-6 by PMDG Simulations v2.0.47.
  - Fuel and service repair still not synced.
- Removed: Numerous profiles of discontinued enhancement mods.

### Aircraft Fixes

- A32NX: AUX fuel tanks not filling through EFB.
- TBM930: Pilot and copilot reading lights.

## Version 2.7.4

### Profiles

- Added: Bell 407 by Microsoft & Nemeth Designs (v0.2.1).
- Added: Cirrus SF50 Vision Jet G2 by FlightFX (v1.2.0).
- Added: Curtiss JN4 Jenny by Microsoft & iniBuilds (v1.1.2).
- Added: Douglas DC-3 by Microsoft & Aeroplane Heaven (v1.0.4).
- Added: DHC-2 Beaver by Microsoft & Blackbird Sims (v1.0.2).
- Added: Grumman G-21A Goose by Microsoft & iniBuilds (v1.1.2).
- Added: Hughes H4 Hercules by Microsoft & Bluemesh (v1.1.3).
- Added: PAC P-750 XSTOL by ORBX (v1.0.1).
- Updated: Aeroplane Heaven C140 to v1.5.0.
- Updated: Aerosoft DHC-6 Twin Otter to v1.1.1.
- Updated: Carenado PC-12 to v1.1.0.
- Updated: DC Designs Concorde to v1.0.6.
- Updated: FlyByWire A32NX (Stable) to v0.9.1.
- Updated: FlyByWire A32NX (Dev/Exp) to December 24, 2022.
- Updated: Flysimware C414AW Chancellor to v3.2.3.
- Updated: Headwind A339X to v0.3.1.
- Updated: LatinVFR A319ceo to v1.0.7.
- Updated: LatinVFR A321neo to v1.2.10.
- Updated: Microsoft Bonanza V35 to v1.6.0.
- Updated: Microsoft Junkers Ju 52 to v0.1.9.
- Updated: Milviz C310R to v1.1.9.
- Updated: Milviz PC-6 to v1.1.2.
- Updated: Salty Simulations 747-8i (Dev) to December 4, 2022.
- Updated: SimSkunkWorks TF-104G Starfighter to v3.3.1.
- Updated: SimWorks Studios Kodiak 100 II & III to v1.4.0.

### Changes

- All aircraft: Removed unsettable circuit failure variables.
- All Carenado: Re-enabled tablet button sync of pilot models.
- Aerosoft DHC-6: Fixed cabin general, reading, emergency, and FO ceiling lights.
- Aerosoft DHC-6: Fixed cockpit caution, entrance, and skydiver day/night lights.
- Big Radials JRF-6B: Fixed manual fuel pump lever.
- FlyByWire A32NX: Attempted to fix glitching SPD selection.
- FlyByWire A32NX: Fixed desync of ISIS SPD/ALT bugs.
- Flysimware C414: Fixed tablet fuel tank spam.
- Milviz C310: Fixed tablet dirt level spam.
- Milviz C310: Fixed tablet fuel tank spam.
- Milviz PC-6: Fixed trapdoor open/close.
- Parallel 42 Kitfox: Fixed pilot & copilot doors.
- SimSkunkWorks TF-104G: Fixed missing fuel sync for center & tip tanks.
- Fixed typo with COM RECEIVE ALL.
- Added support for NAV3 and NAV4.
- Added support for rotorcraft.

## Version 2.7.3

### Profiles

- Added: Savoia-Marchetti S.55 & S.55X by Microsoft (v1.5.3).
- Added: Tecnam P92 Echo by Erasam (v1.5).
- Corrected aircraft-dependent profiles for latest name changes. (e.g. Heavy Division 787)

### Aircraft Fixes

- Asobo Longitude: Added more payload to support Dakfly mod.
- FlyByWire A32NX: Reverted some changes that broke AP knobs in last update.

### Issues
![image](https://cdn.discordapp.com/attachments/765289974232252466/1028444695069737030/fbwkeepoff.png)

## Version 2.7.2

### Profiles

- Added: BN-2 Islander by BlackBox Simulation (v2.1.2).
- Added: BN-3 Trislander by BlackBox Simulation (v1.1.6).
- Added: Cessna C195 Businessliner by Microsoft (v1.4.2).
- Added: Cessna C510 Mustang by Cockspur (v1.0.3).
- Updated: FlyByWire A32NX (Dev/Exp) to October 2nd, 2022.

### Aircraft Fixes

- FlyByWire A32NX: Improved sync of ground services on tablet.

## Version 2.7.1

### Profiles

- Complete profile re-organization.
- Added: Airbus A319ceo by LatinVFR (v1.0.4).
- Added: Airbus A321neo by LatinVFR (v1.2.5).
- Added: Beechcraft Bonanza V35 by Microsoft (v1.5.2).
- Added: Beechcraft Model 17 Staggerwing by Microsoft (v1.8.0).
- Added: Beechcraft Model 18 by Microsoft (v1.9.0).
- Added: Cessna CT182T Skylane by Carenado (v1.5.0).
- Added: Cessna C337 Skymaster by Carenado (v1.1.2).
- Added: Discus2c by GotFriends (v2.0.4).
- Added: Edgley EA-7 Optica by GotFriends (v1.0.3).
- Added: Hondajet by FlightFX (v1.0.6).
  - WT G3000 currently not supported, setting route won't sync.
- Added: Junkers F 13 by Microsoft (v0.1.1).
- Added: Junkers Ju 52 by Microsoft (v0.1.8).
- Added: Pilatus PC-12 by Carenado (v1.0.3).
- Added: Piper PA-28R Arrow III by Carenado (v1.5.0).
- Added: Piper PA-44 Seminole by Carenado (v1.8.0).
- Added: Schleicher AS33Me by MADolo Simulations (1.51).
- Updated: Flysimware C414AW Chancellor to v3.2.0.
- Updated: SimWorks Studios RV-14 to v1.3.2.

### Aircraft Fixes

- FlyByWire A32NX: Removed possible duplicate pushback command causing no more sync.
- Carenado M20R: Desync with vertical speed gauge.
- Carenado Seneca V: Landing pulse lights not turning off.

## Version 2.7.0

### Program Fixes

- Performance degradation with community module on solo flights longer than 2 hours.
- H-event duplication after ending and making new server with someone previously connected.
- An issue preventing FlyByWire A32NX compatiblity with Cloud Host.
- An issue with definitions not being received by clients through Cloud Host.
- Attempted to fix an issue which prevented those behind CGN (Carrier-grade NAT) to connect through Cloud Host.
- Systems on LAN can now connect through Cloud P2P.
- Added message for when cloud hoster connection is lost. First client in list becomes new hoster.
- Added hide clickability for IP addresses and session code.
- Added IPv6 support for Cloud P2P and Cloud Host.
- Hosting via the `Direct` method will now listen on both IPv6 and IPv4 addresses.
- Current session ID moved to the `Server` tab.

### Profiles

- Added: A32NX Stable by FlyByWire (v0.8.1).
- Added: Bell 47G by FlyInside (v1.71).
- Added: Boeing 247D by Wing42 (v1.0.1).
- Added: C140 by Aeroplane Heaven (v1.4.0a).
- Added: C170B by Carenado (v1.3.0).
- Added: C208B EX Improvement mod by Magraina (v2203.2.2).
- Added: C310R by Milviz (1.0.0).
- Added: C414AW Chancellor by Flysimware (v2.7.0).
- Added: Concorde by DC Designs (v1.0.4).
- Added: DHC6 Twin Otter by Aerosoft (v1.0.6.0).
- Added: Electra-10A by Aeroplane Heaven (v1.33.7).
- Added: FreedomFox & Fox2 (KitFox STi) by Parallel 42 (v1.0.0).
- Added: G115 Tutor T.1 by IRIS Simulations (v2.3.7).
- Added: H125 by RotorSimPilot (v1.3.9).
- Added: J160/J170 Jabiru by IRIS Simulations (v1.5.6).
- Added: JRF-6B Goose by Big Radials (v1.0.4).
- Added: Kodiak 100 by SimWorks Studios (v1.2.2).
- Added: PA-28 Warrior II by JustFlight (v0.3.5).
- Added: PA-28R Turbo Arrow III/IV by JustFlight (v0.5.5).
- Added: PC-6 Turbo Porter by Milviz (v1.0.9).
- Added: RV-14 & 14A by SimWorks Studios (v1.2.0).
- Added: RV-7 & 7A mod by Deejing (v1.0.8).
- Added: Stream by LightSim (v1.3).
- Added: TF-104G Starfighter by SimSkunkWorks (v3.1.2).
- Added: Basic support for GNS530 by PMS50 (v1.0.50).
- Added: Basic support for GTN750 by PMS50 (v2.1.41).
- Added: Basic support for GTNXi 750 by TDS Sim Software (v1.0.1.8).
- Added: Basic support for G1000 NXi by Working Title (v1.0.1).
- Updated: ASOBO aircraft for Sim Update 9 (v1.25.7.0).
- Updated: Carenado M20R to v1.5.1.
- Updated: Carenado Seneca V to v1.4.0.
- Updated: FlyByWire A32NX to May 18, 2022.
  - EFB not updated.
- Updated: Headwind A330-900 to v0.200.
- Updated: HypePerformanceGroup H135 to v1.4.5.
- Updated: JustFlight Arrow III to v0.10.5.
- Updated: Mrtommymxr C172 to v0.3.
- Updated: Mrtommymxr DA62X to v0.7.
- Updated: Mrtommymxr DA40NGX to v0.8.6.
- Updated: RotorSimPilot R44 to v1.2.8.
- Updated: SaltySimulations 747 to v0.5.1.
- Updated: Working Title CJ4 to v0.12.13.
- Removed: Aerosoft CRJ (being re-evaluated).
- Removed: Frett G36 (deprecated).
- Removed: PMDG DC-6 (being re-created).

### General Fixes

- All autopilot and radio button/knob desync on aircraft utilizing legacy simvars.
- Avionics master switches on all aircraft experiencing intermittent issues.
- Master caution and warning acknowledge events since Sim Update 7.
- Autopilot automatically leveling off when using V/S and FLC.
- Inaccurate G1000 COM/NAV volume level percentage.
- NAV/ADF volume knobs jumping back and forth.
- ADF frequency not always swapping.
- Parking brake event name spelling error.
- Attitude indicator bar calibration.
- Annunciator test light switch.
- Engine bleed air toggle event.
- External power toggle event.
- Propeller condition lever.
- Added pushback tug support.

### Aircraft Fixes

- Asobo 747: ATC ground services.
- Asobo 787: ATC ground services.
- Asobo A320neo: ATC ground services.
- Asobo A320neo: WX brightness knob.
- Asobo C172: Kohlsman index for G1000 PFD.
- Asobo C208: External power BUS and STBY alternator.
- Asobo Cap10: Missing external lights.
- Asobo Cap10: Aerobatic trim flap switch.
- Asobo CJ4: ATC ground services.
- Asobo CJ4: Pulse light pushbutton.
- Asobo KingAir: Added 3rd interior cabin light.
- Asobo Longitude: ATC ground services.
- Asobo Longitude: SVT terrain toggle desync.
- Asobo Porter: Profile not executing.
- Asobo Porter: Support for non-G950 variant.
- Asobo Porter: Master battery and avionics switches.
- Asobo Porter: Landing lights.
- Asobo SW121: Missing panel potentiometer.
- Asobo TBM930: SVT terrain toggle desync.
- A32NX: AP1 to AP2 toggle with a temporary workaround.
- A32NX: Autobrake level setting (#7067).
- A32NX: A/THR turning off and back on.
- A32NX: Corrected 16K value for flap handle.
- A32NX: Gear lever for new hydraulic system (#6893).
- A32NX: Ground services including exit and cargo doors (#7229).
- A32NX: LS pushbutton desync.
- A32NX: Mach airspeed knob.
- Carenado M20R: Interior and exterior lights.
- Carenado Seneca V: Added TDS GTN support.
- Headwind A339: ATC ground services.
- HPG H135: Barometer desync with MFD knobs.
- HPG H135: Excessive data spam while in hover mode.
- HPG H135: Tablet autopilot button desync.
- RSP H125: Added TDS GTN support.
- RSP R44: Added TDS GTN support.
- Salty 747: ATC ground services.
- Salty 747: Flight director switches.
- Salty 747: Sync of LNAV/VNAV modes.
- Working Title CJ4: Vertical pitch reference indicator.

### New issues

- B-events/variables: This is a new system introduced by Asobo with Sim Update 5. All switches and levers now using "B:" no longer visually move but any associated "A:" or "L:" secondary vars will still sync, causing proper effect. This is because "B:" is not settable through SimConnect. Asobo currently shows no intent on fixing this.
- Touchscreen interaction on 3rd-party avionics packages currently does not sync. This is a 3rd-party WASM limitation. Full support for such packages may or may not be possible.
- JustFlight PA-28 Bundle: Avionics power trouble may occur during cold start if GPS type on tablet are not the same for all connected. Each person should cycle type before powering.
- Milviz C310R: Some actions on tablet only sync once everyone connected has tablet visible.
- FlyByWire current fuel state unsyncable.