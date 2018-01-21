import { SingleEvent } from "./display-event";
import { Component, Input } from "@angular/core";

@Component({
    selector: '[display-event-row]',
    templateUrl: './display-event-row.html',
    styleUrls: ['./events.component.scss']
  })
export class DisplayEventRowComponent {
    @Input() event: SingleEvent;
    public constructor() { }

}