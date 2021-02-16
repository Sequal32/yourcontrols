from zipfile import ZipFile, ZIP_DEFLATED
from os.path import basename
import os

def insertAllIntoZip(zipObj, directory):
    # Iterate over all the files in directory
   for folderName, subfolders, filenames in os.walk(directory):
        for filename in filenames:
            #create complete filepath of file in directory
            filePath = os.path.join(folderName, filename)
            # Add file to zip
            zipObj.write(filePath)

zipObj = ZipFile("scripts/out/YourControls.zip", "w", ZIP_DEFLATED)

insertAllIntoZip(zipObj, "definitions/aircraft")
insertAllIntoZip(zipObj, "definitions/modules")
zipObj.write("assets/logo.png", "assets/logo.png")
zipObj.write("target/release/YourControls.exe", "YourControls.exe")
zipObj.write("SimConnect.dll", "SimConnect.dll")

zipObj.close()