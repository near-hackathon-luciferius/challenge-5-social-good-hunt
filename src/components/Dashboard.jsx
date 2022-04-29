import React from 'react';
import PropTypes from 'prop-types';
import { Link } from 'react-router-dom';

const Dashboard = ({version, currentUser}) => {
   return <>
                 <header>
                   <h1>NEAR Challenge #5 - Social Bounty Hunt - {version}</h1>
                 </header>
                  <p>
                      This app was developed for the NEAR Spring hackathon.
                  </p>
                  <p>
                      Here is how the app works. Everyone can publish the good social deeds they performed in order to earn DEED.
                      They publish they social deed with a description and a proof (for now this must be a link to an image or gif).
                      Others can now credit the social deed that was published. For each credit the author of the social deed gets
                      rewarded with one DEED.
                  </p>
                  <p>
                      DEED is a non-transferable and therefore non-tradable fungible token. Is represents the users social reputation
                      for his good deeds. But there is a second utility to the token. Users can also donate NEAR to the app. When they
                      do that, the donated NEAR is distributed to all DEED holders proprtional to they DEED amount. The donation itself
                      will automatically create a new deed for the donator.
                  </p>
                  <p>
                      They idea behind that was, that there are two kinds of users. Those who do good social deeds. Those who want to
                      promote themselfs by donating. For example a new crypto project can donate to the app. With the donation the
                      crypto project would get recognition from all users.
                  </p>
                  <p>
                      Of course in a real world scenario all users must be KYCed in order to prevent gaming the system by creating a lot
                      of new accounts and accumulating a lot of DEED.
                  </p>
                 <h5>Head over <Link className="menu-item" to="/publish">here</Link> to publish your first social 
                     deed. Or look at social deeds others have published so far <Link className="menu-item" to="/overview">here</Link>.
                     Lastly you can <Link className="menu-item" to="/donate">donate</Link> NEAR to all DEED holders and automatically create a deed with that.
                 </h5>
             </>
}

Dashboard.propTypes = {
  version: PropTypes.string.isRequired,
  currentUser: PropTypes.shape({
    accountId: PropTypes.string.isRequired,
    balance: PropTypes.string.isRequired
  })
};

export default Dashboard;