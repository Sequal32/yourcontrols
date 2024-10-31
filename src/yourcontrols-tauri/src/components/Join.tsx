import React from 'react'
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';

const Join: React.FC = () => (
    <Card className='h-full'>
        <CardHeader>
            <CardTitle>Join</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
            <div className="space-y-1">
                <Label htmlFor="name">Name</Label>
                <Input id="name" defaultValue="Pedro Duarte" />
            </div>
            <div className="space-y-1">
                <Label htmlFor="username">Username</Label>
                <Input id="username" defaultValue="@peduarte" />
            </div>
        </CardContent>
        <CardFooter>
            <Button>Save changes</Button>
        </CardFooter>
    </Card >
);


export default Join;