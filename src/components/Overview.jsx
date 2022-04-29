import React, { useState, useEffect } from 'react';
import PropTypes from 'prop-types';
import { Button } from 'react-materialize';

const Overview = ({currentUser, contract, onCredit}) => {
    const [deeds, setDeeds] = useState([]);
  
  useEffect(() => {
      async function fetchData() {
          const count = await contract.get_deeds_count();
          const result = await contract.social_deeds(
          {
              creditor_id: currentUser.accountId,
              from_index: "0",
              limit: parseInt(count)
          });
          console.log(result);
          setDeeds(splitArrayIntoChunksOfLen(result, 2));
      }
      
      fetchData();
  }, [contract, currentUser]);
  
  const splitArrayIntoChunksOfLen = (arr, len) => {
    var chunks = [], i = 0, n = arr.length;
    while (i < n) {
      chunks.push(arr.slice(i, i += len));
    }
    return chunks;
  }
  
   return <>
                 <header>
                   <h1>All deeds that were published.</h1>
                 </header>
                 
                  {deeds.map(chunk => 
                  <div className="row">
                    {chunk.map(deed =>
                      <div className="col s6" key={deed.id}>
                          <div className="card">
                            <div className="card-image">
                              <img src={deed.proof} alt={deed.proof}/>
                            </div>
                            <div className="card-title">{deed.title}</div>
                            <div className="card-content">
                              <p><b>Author: {deed.author}</b></p>
                              <p>{deed.description}</p>
                              <p>{deed.is_creditor}</p>
                            </div>
                            <div className="card-action">
                              <span className='important'>{deed.creditors}</span>
                              {
                                deed.is_creditor
                                ?<Button small
                                        tooltip="You already credited the author."
                                        className="margin_button disabled">
                                  Credit
                                </Button>
                                : deed.author === currentUser.accountId
                                  ? <Button small
                                            tooltip="You cannot credit yourself."
                                            className="margin_button disabled">
                                      Credit
                                    </Button>
                                  : <Button onClick={() => onCredit(deed)} small
                                            tooltip="Give a credit to the deed author."
                                            className="margin_button">
                                      Credit
                                    </Button>
                              }
                            </div>
                          </div>
                      </div>)}          
                  </div>)}
          </>
}

Overview.propTypes = {
  currentUser: PropTypes.shape({
    accountId: PropTypes.string.isRequired,
    balance: PropTypes.string.isRequired
  }),
  contract: PropTypes.shape({
    get_deeds_count: PropTypes.func.isRequired,
    social_deeds: PropTypes.func.isRequired
  }).isRequired,
};

export default Overview;