import { FsmEvent } from "../fsm-event";
import { Component, Input } from "@angular/core";

export type DisplayEvent = SingleEvent | TransitionEvents;

export class SingleEvent {
    constructor(public event: FsmEvent) { }
}

export class TransitionEvents {
    constructor(public events: Array<FsmEvent>) { }
}


@Component({
    selector: '[displayEvent]',
    templateUrl: './display-event.html',
    styleUrls: ['./events.component.scss']
  })
export class DisplayEventComponent {

    @Input() event: DisplayEvent;
  
    public constructor() { }

}