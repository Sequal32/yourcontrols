from zipfile import ZipFile
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

zipObj = ZipFile("out/YourControls.zip", "w")

zipObj.write("../assets/logo.png", "assets/logo.png")
zipObj.write("../target/release/YourControls.exe", "YourControls.exe")
zipObj.write("../definitions/", "definitions")
zipObj.write("../SimConnect.dll", "SimConnect.dll")