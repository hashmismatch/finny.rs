import { Component } from '@angular/core';
import { OnInit } from '@angular/core/src/metadata/lifecycle_hooks';
import { EventsComponent } from './events/events.component';
import { FsmEventStateTransitioned, FsmEventProcessingEvent, FsmEventState, FsmEventAction } from './fsm-event';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent {
  
  constructor() {

  }
  title = 'app';
}
