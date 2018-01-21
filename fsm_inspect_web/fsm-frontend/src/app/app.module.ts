import { BrowserModule } from '@angular/platform-browser';
import { NgModule } from '@angular/core';


import { AppComponent } from './app.component';
import { EventsComponent } from './events/events.component';
import { FsmOverviewComponent } from './fsm-overview/fsm-overview.component';

import {HttpClientModule} from '@angular/common/http';
import { FsmStateService } from './fsm-state.service';
import { AngularSplitModule } from 'angular-split';
import { JsonInspectComponent } from './json-inspect/json-inspect.component';
import {Tabs} from './tabs';
import {Tab} from './tab';
import { DisplayEventComponent } from './events/display-event';
import { DisplayEventRowComponent } from './events/display-event-row';

@NgModule({
  declarations: [
    AppComponent,
    EventsComponent,
    DisplayEventComponent,
    DisplayEventRowComponent,
    FsmOverviewComponent,
    JsonInspectComponent,

    Tabs,
    Tab
  ],
  imports: [
    BrowserModule,
    HttpClientModule,
    AngularSplitModule
  ],
  providers: [
    FsmStateService
  ],
  bootstrap: [AppComponent]
})
export class AppModule { }
