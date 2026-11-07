import {mount} from 'svelte';
import App from './App.svelte';
import 'devices.css/dist/devices.css';
import './app.css';

mount(App, {target: document.getElementById('app')!});
