import React from 'react'
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { ToggleGroup, ToggleGroupItem } from './ui/toggle-group';

const Host: React.FC = () => (
    <Card>
        <CardHeader>
            <CardTitle>Host</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">

            <div className="flex flex-row items-center justify-between rounded-lg border p-3 shadow-sm">
                <p>IP Version</p>
                <ToggleGroup type="single" defaultValue='ipv4' variant="outline">
                    <ToggleGroupItem value="ipv4">
                        IPv4
                    </ToggleGroupItem>
                    <ToggleGroupItem value="ipv6">
                        IPv6
                    </ToggleGroupItem>
                </ToggleGroup>
            </div>

            <div className="flex flex-row items-center justify-between rounded-lg border p-3 shadow-sm">
                <p>Hosting Mode</p>
                {/* TODO: merge buttons with css */}
                <ToggleGroup type="single" defaultValue='p2p' variant="outline">
                    <ToggleGroupItem value="p2p">
                        Cloud P2P
                    </ToggleGroupItem>
                    <ToggleGroupItem value="host">
                        Cloud Host
                    </ToggleGroupItem>
                    <ToggleGroupItem value="direct">
                        Direct
                    </ToggleGroupItem>
                </ToggleGroup>
            </div>

        </CardContent>
        <CardFooter>
            <Button className="w-full">Start Server</Button>
        </CardFooter>
    </Card>
);


export default Host;