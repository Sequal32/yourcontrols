import React, { useEffect, useMemo, useState } from 'react'
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectTrigger, SelectValue } from './ui/select';
import { Tooltip, TooltipContent, TooltipTrigger } from './ui/tooltip';
import { Switch } from './ui/switch';
import { useForm } from 'react-hook-form';
import { Form, FormControl, FormField, FormItem, FormMessage } from './ui/form';
import { Input } from './ui/input';
import clsx from 'clsx';
import { Separator } from './ui/separator';
import { AircraftConfig, commands } from '@/types/bindings';

const Settings: React.FC = () => {
    // TODO: add zod validation
    const form = useForm();

    const [aircraftConfigs, setAircraftConfigs] = useState<Awaited<ReturnType<typeof commands.getAircraftConfigs>>>();

    useEffect(() => {
        commands.getAircraftConfigs()
            .then((payload) => {
                console.log('getAircraftConfigs', payload);
                setAircraftConfigs(payload);
            })
            .catch((error) => {
                console.error('getAircraftConfigs', error);
            });
    }, []);

    const onSubmit = (data: any) => {
        console.log('onSubmit', data);
    }

    return (
        <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)}>
                <Card>
                    <CardHeader>
                        <CardTitle>Settings</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                        <FormField
                            control={form.control}
                            name="username"
                            render={({ field }) => (
                                <div className="flex justify-between items-center rounded-lg border p-4">
                                    <p className='font-medium capitalize'>{field.name}</p>
                                    <FormItem className='w-2/3'>
                                        <FormControl>
                                            <Input placeholder="Captain123" autoComplete="off" {...field} />
                                        </FormControl>
                                        <FormMessage />
                                    </FormItem>
                                </div>
                            )}
                        />

                        <FormField
                            control={form.control}
                            name="aircraft"
                            render={({ field }) => (
                                <div className="flex justify-between items-center rounded-lg border p-4">
                                    <p className="font-medium">Aircraft</p>
                                    <Select onValueChange={field.onChange} defaultValue={field.value} disabled={!aircraftConfigs}>
                                        <FormControl>
                                            <SelectTrigger className={clsx("w-2/3", !field.value && "text-muted-foreground")} >
                                                <SelectValue placeholder="..." />
                                            </SelectTrigger>
                                        </FormControl>
                                        <SelectContent>
                                            {aircraftConfigs && Object.entries(aircraftConfigs).map(([category, aircraftArray]) => (
                                                <SelectGroup key={category} className='-m-1 group select-none'>
                                                    <Separator className='group-first:hidden' />
                                                    <SelectLabel className='dark:bg-white/5'>{category}</SelectLabel>
                                                    <Separator />
                                                    {aircraftArray.map(aircraftConfig => (
                                                        <SelectItem key={category + aircraftConfig.name} value={aircraftConfig.path}>{aircraftConfig.name}</SelectItem>
                                                    ))}
                                                </SelectGroup>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                    <FormMessage />
                                </div>
                            )}
                        />

                        <div className="flex items-center space-x-4 rounded-lg border p-4">
                            <div className="flex-1 space-y-1">
                                <p className="font-medium leading-none">
                                    Instructor Mode
                                </p>
                                <p className="text-sm text-muted-foreground">
                                    New connections will be automatically placed in observer mode.
                                </p>
                            </div>
                            <Switch />
                        </div>

                        <div className="flex items-center space-x-4 rounded-lg border p-4">
                            <div className="flex-1 space-y-1">
                                <p className="font-medium leading-none">
                                    Streamer Mode
                                </p>
                                <p className="text-sm text-muted-foreground">
                                    Hides your IP and the session code after connecting.
                                </p>
                            </div>
                            <Switch />
                        </div>
                    </CardContent>
                    <CardFooter >
                        <Tooltip delayDuration={100}>
                            <TooltipTrigger asChild>
                                <Button type='submit' variant="constructive" className="w-full">Save Settings</Button>
                            </TooltipTrigger>
                            <TooltipContent side='bottom'>
                                Save Changes For Next Session
                            </TooltipContent>
                        </Tooltip>
                    </CardFooter>
                </Card>
            </form>
        </Form>
    )
};

export default Settings;
