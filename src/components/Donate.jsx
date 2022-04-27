import React from 'react';
import { Button, TextInput } from 'react-materialize';

export default function Donate({onDonate}) {
  return (
    <>
      <header>
                   <h1>Donate NEAR.</h1>
      </header>
      <form onSubmit={onDonate}>
        <fieldset id="fieldset">
            <p>Here you can donate NEAR.</p>
            <p>
                When you hit Donate the chosen amount of NEAR will be distributed to all DEED holders.
                This excludes your own account as you are doing the donation. Secondly a new deed is 
                automatically created which shows that you donated the chosen amount.
            </p>
            <p className="highlight">
            <TextInput
                autoComplete="off"
                id="donation"
                defaultValue={'1'}
                max="100"
                min="1"
                step="1"
                type="number"
                label="Add the amount of Ⓝ you want to donate."
                required
            >
            </TextInput>
            </p>
            <Button type="submit" small
                    tooltip="Donates the chosen amount of Ⓝ.">
              Donate
            </Button>
        </fieldset>
      </form>
    </>
  );
}