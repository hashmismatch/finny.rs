import { Component, OnInit } from '@angular/core';
import { Observable } from 'rxjs/Observable';
import { BehaviorSubject } from 'rxjs/BehaviorSubject';
import { FsmEvent, FsmEventStateTransitioned, FsmEventActionExecuted, FsmEventProcessingEvent, FsmEventState, FsmEventAction } from '../fsm-event';
import { FsmStateService } from '../fsm-state.service';
import { ViewChild } from '@angular/core';
import { ElementRef } from '@angular/core';
import { DisplayEvent, SingleEvent } from './display-event';

@Component({
  selector: 'app-events',
  templateUrl: './events.component.html',
  styleUrls: ['./events.component.scss']
})
export class EventsComponent implements OnInit {

  @ViewChild('eventsTableBody') eventsTableBody: ElementRef;

  public autoScroll: boolean = true;

  private displayEventsList: Array<DisplayEvent> = new Array();

  constructor(private fsm: FsmStateService) {
    
  }

  ngOnInit() {
    // todo: combining events
    this.fsm.observableFsmEvents.subscribe((event) => {
      this.displayEventsList.push(new SingleEvent(event));
    });
  }

  
  private startedAutoScroll: boolean = false;
  private lastEventCount: number = 0;

  ngAfterViewChecked() {
    if (this.autoScroll && !this.startedAutoScroll) {
      var isScrollable = this.eventsTableBody.nativeElement.scrollHeight > this.eventsTableBody.nativeElement.clientHeight;
      if (this.lastEventCount != this.fsm.getEventCount() && isScrollable) {
        this.lastEventCount = this.fsm.getEventCount();
        this.startedAutoScroll = true;
        this.eventsTableBody.nativeElement.scrollTop = 999999999;        
      }
    }
  }

  ngAfterViewInit() {    
    this.eventsTableBody.nativeElement.addEventListener("scroll", (ev: Event) => {
      if (this.startedAutoScroll == false) {
        if (this.autoScroll == false) {
          // should we re-enable autoscroll if the user went to the bottom of the event log?
          var d = this.eventsTableBody.nativeElement.scrollHeight - this.eventsTableBody.nativeElement.scrollTop;
          // tolerance when auto-scroll starts
          if (Math.abs(this.eventsTableBody.nativeElement.clientHeight - d) <= 20) {
            this.autoScroll = true;
          }
        } else {
          // disable auto scroll
          this.autoScroll = false;
        }
      }
      this.startedAutoScroll = false;
    });
  }

  trackByEvent(index: number, item: any): number {
    if (item instanceof SingleEvent) {
      return item.event.id;
    }

    return index;
  }
  

}
